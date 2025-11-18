# LLM-Registry Architecture

## Table of Contents
1. [System Overview](#system-overview)
2. [Architecture Layers](#architecture-layers)
3. [Rust Crate Recommendations](#rust-crate-recommendations)
4. [Deployment Architectures](#deployment-architectures)
5. [Integration Architecture](#integration-architecture)
6. [Data Flow Architecture](#data-flow-architecture)

---

## System Overview

LLM-Registry serves as the centralized metadata repository and discovery service for the LLM ecosystem. It provides a high-performance, type-safe registry for models, datasets, prompts, and their associated metadata with support for embedded, standalone, and distributed deployment models.

### Core Design Principles

- **Performance First**: Rust-native implementation with zero-copy serialization
- **Type Safety**: Strong typing throughout the stack with compile-time guarantees
- **Flexibility**: Support multiple deployment modes from embedded to distributed
- **Observability**: Built-in metrics, tracing, and health monitoring
- **Extensibility**: Plugin architecture for custom validators and transformers

### System Component Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          CLIENT APPLICATIONS                             │
│  (LLM-Forge SDK, Governance Dashboard, Memory Graph, Policy Engine)     │
└────────────────┬────────────────────────────────────┬───────────────────┘
                 │                                    │
                 │ REST/GraphQL/gRPC                  │ WebSocket (Events)
                 │                                    │
┌────────────────▼────────────────────────────────────▼───────────────────┐
│                           API LAYER                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │
│  │ REST API     │  │ GraphQL API  │  │ gRPC Service │  │ WebSocket   │ │
│  │ (axum)       │  │ (async-gql)  │  │ (tonic)      │  │ (tokio-ws)  │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬──────┘ │
│         │                 │                 │                 │         │
│         └─────────────────┴─────────────────┴─────────────────┘         │
│                                   │                                     │
└───────────────────────────────────┼─────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼─────────────────────────────────────┐
│                        SERVICE LAYER                                     │
│  ┌───────────────────┐  ┌──────────────────┐  ┌────────────────────┐   │
│  │ Registry Service  │  │ Validation Svc   │  │ Replication Svc    │   │
│  │ - CRUD ops        │  │ - Schema check   │  │ - Change detection │   │
│  │ - Versioning      │  │ - Policy eval    │  │ - Sync manager     │   │
│  │ - Lineage track   │  │ - Metadata enrich│  │ - Conflict resolve │   │
│  └─────────┬─────────┘  └────────┬─────────┘  └──────────┬─────────┘   │
│            │                     │                        │             │
│  ┌─────────▼─────────────────────▼────────────────────────▼─────────┐   │
│  │                    Transaction Coordinator                        │   │
│  │                    (ACID guarantees, 2PC)                         │   │
│  └─────────┬─────────────────────┬────────────────────────┬─────────┘   │
└────────────┼─────────────────────┼────────────────────────┼─────────────┘
             │                     │                        │
┌────────────▼─────────────────────▼────────────────────────▼─────────────┐
│                          DATA LAYER                                      │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────────┐  │
│  │ Primary Storage  │  │ Search Index     │  │ Cache Layer          │  │
│  │ (sled/redb)      │  │ (tantivy)        │  │ (moka/mini-moka)     │  │
│  │ - Versioned KV   │  │ - Full-text      │  │ - LRU/TTL eviction   │  │
│  │ - ACID txns      │  │ - Faceted search │  │ - Multi-tier cache   │  │
│  │ - Snapshots      │  │ - Ranking        │  │ - Invalidation       │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────────┘  │
│                                                                          │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────────┐  │
│  │ Metadata Store   │  │ Blob Storage     │  │ Replication Log      │  │
│  │ (sled trees)     │  │ (filesystem/S3)  │  │ (append-only log)    │  │
│  │ - Schema defs    │  │ - Model weights  │  │ - Change stream      │  │
│  │ - Lineage graph  │  │ - Large artifacts│  │ - Event sourcing     │  │
│  │ - Relationships  │  │ - Checkpoints    │  │ - Time travel        │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼─────────────────────────────────────┐
│                      INTEGRATION LAYER                                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐ │
│  │ Memory Graph    │  │ Policy Engine   │  │ Forge SDK               │ │
│  │ Connector       │  │ Connector       │  │ Connector               │ │
│  │ - Graph sync    │  │ - Policy hooks  │  │ - Metadata sync         │ │
│  │ - Lineage push  │  │ - Validation CB │  │ - Bidirectional updates │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────┘ │
│                                                                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐ │
│  │ Governance Dash │  │ External APIs   │  │ Event Publishers        │ │
│  │ Connector       │  │ (HuggingFace)   │  │ (Kafka/NATS/Redis)      │ │
│  │ - Analytics feed│  │ - Model import  │  │ - Change notifications  │ │
│  │ - Admin APIs    │  │ - Sync adapters │  │ - Audit stream          │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│                    OBSERVABILITY & INFRASTRUCTURE                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐  │
│  │ Metrics      │  │ Tracing      │  │ Logging      │  │ Health      │  │
│  │ (prometheus) │  │ (opentelemetry│  │ (tracing)   │  │ Checks      │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  └─────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Architecture Layers

### 1. API Layer

The API Layer provides multiple protocol interfaces for client interaction, designed for flexibility and performance.

#### Components

**REST API (Primary Interface)**
- **Framework**: `axum` 0.7+ with Tower middleware
- **Features**:
  - RESTful endpoints for CRUD operations
  - JSON/MessagePack content negotiation
  - Request validation and sanitization
  - Rate limiting and throttling
  - API versioning (URL and header-based)
  - OpenAPI 3.1 specification generation
- **Endpoints**:
  - `/v1/models` - Model registry operations
  - `/v1/datasets` - Dataset catalog
  - `/v1/prompts` - Prompt templates
  - `/v1/metadata` - Generic metadata queries
  - `/v1/search` - Full-text and faceted search
  - `/v1/lineage` - Lineage tracking and queries

**GraphQL API (Complex Queries)**
- **Framework**: `async-graphql` 7.0+
- **Features**:
  - Type-safe schema with derived types
  - Dataloader for N+1 query optimization
  - Subscription support for real-time updates
  - Field-level authorization
  - Query complexity analysis
  - Batching and caching
- **Schema Highlights**:
  ```graphql
  type Model {
    id: ID!
    name: String!
    version: String!
    metadata: JSON!
    lineage: [LineageNode!]!
    relationships: ModelRelationships!
  }

  type Query {
    model(id: ID!): Model
    searchModels(filter: ModelFilter!, pagination: Pagination!): ModelConnection!
    lineageGraph(rootId: ID!, depth: Int): LineageGraph!
  }

  type Mutation {
    registerModel(input: ModelInput!): Model!
    updateMetadata(id: ID!, metadata: JSON!): Model!
  }

  type Subscription {
    modelUpdated(id: ID!): Model!
    registryEvents(filter: EventFilter): RegistryEvent!
  }
  ```

**gRPC Service (High-Performance)**
- **Framework**: `tonic` 0.12+ with `prost` for codegen
- **Features**:
  - Binary protocol for low-latency communication
  - Streaming support (client, server, bidirectional)
  - Health checking protocol
  - Reflection for dynamic clients
  - TLS with mutual authentication
  - Load balancing support
- **Services**:
  ```protobuf
  service RegistryService {
    rpc GetModel(GetModelRequest) returns (Model);
    rpc RegisterModel(RegisterModelRequest) returns (Model);
    rpc StreamModels(StreamModelsRequest) returns (stream Model);
    rpc SyncMetadata(stream MetadataUpdate) returns (stream SyncResponse);
  }
  ```

**WebSocket (Real-Time Events)**
- **Framework**: `tokio-tungstenite` with `axum` integration
- **Features**:
  - Subscribe to registry change events
  - Bi-directional command/response
  - Automatic reconnection support
  - Per-connection filtering and permissions
  - Heartbeat/keepalive

#### Cross-Cutting Concerns

- **Authentication**: JWT validation, API key verification
- **Authorization**: Role-based access control (RBAC)
- **Rate Limiting**: Token bucket algorithm per client
- **CORS**: Configurable origin policies
- **Compression**: Gzip/Brotli for response compression
- **Request Tracing**: Distributed tracing with OpenTelemetry

---

### 2. Service Layer

The Service Layer contains business logic, validation, and orchestration.

#### Registry Service

**Responsibilities**:
- Model/dataset/prompt lifecycle management
- Version control and history tracking
- Metadata enrichment and transformation
- Lineage graph construction
- Relationship management

**Key Operations**:
```rust
pub trait RegistryService {
    async fn register<T: Artifact>(&self, artifact: T) -> Result<ArtifactId>;
    async fn get<T: Artifact>(&self, id: &ArtifactId) -> Result<Option<T>>;
    async fn update<T: Artifact>(&self, id: &ArtifactId, update: Update<T>) -> Result<T>;
    async fn delete(&self, id: &ArtifactId) -> Result<()>;
    async fn list_versions(&self, id: &ArtifactId) -> Result<Vec<Version>>;
    async fn get_lineage(&self, id: &ArtifactId, depth: u32) -> Result<LineageGraph>;
}
```

**Versioning Strategy**:
- Semantic versioning for artifacts
- Copy-on-write for metadata updates
- Version pinning and range resolution
- Immutable version history

#### Validation Service

**Responsibilities**:
- Schema validation against JSON Schema/protobuf definitions
- Policy enforcement integration with LLM-Policy-Engine
- Metadata completeness checks
- Data quality validation
- Custom validation plugin execution

**Validation Pipeline**:
```
Input → Schema Validation → Policy Evaluation → Quality Checks → Enrichment → Output
         (serde_valid)        (policy-engine)    (custom)        (metadata)
```

**Validation Rules**:
- Required field presence
- Type constraints and ranges
- Format validation (URLs, emails, UUIDs)
- Cross-field dependencies
- Business rule enforcement
- Quota and limit checks

#### Replication Service

**Responsibilities**:
- Change detection and capture (CDC)
- Multi-node synchronization
- Conflict resolution strategies
- Consistency verification
- Backup and restore

**Replication Modes**:
- **Leader-Follower**: Single writer, multiple readers
- **Multi-Leader**: Conflict resolution with CRDTs or last-write-wins
- **Leaderless**: Quorum-based reads/writes (Dynamo-style)

**Change Capture**:
```rust
pub struct ChangeEvent {
    pub id: EventId,
    pub timestamp: Timestamp,
    pub artifact_id: ArtifactId,
    pub change_type: ChangeType, // Insert, Update, Delete
    pub before: Option<Metadata>,
    pub after: Option<Metadata>,
    pub causality_token: VectorClock,
}
```

#### Transaction Coordinator

**Responsibilities**:
- ACID transaction management
- Two-phase commit for distributed operations
- Saga pattern for long-running workflows
- Deadlock detection and resolution

**Transaction Boundaries**:
- Single-artifact updates: Local transactions
- Cross-artifact operations: Distributed transactions
- Bulk imports: Batch transactions with checkpointing

---

### 3. Data Layer

The Data Layer provides persistent storage, indexing, and caching.

#### Primary Storage

**Key-Value Store Options**:

1. **sled** (Recommended for most deployments)
   - Embedded B-tree database
   - ACID transactions with MVCC
   - Zero-copy reads
   - Snapshot support
   - ~40K writes/sec, ~1M reads/sec (SSD)

2. **redb** (Alternative for extreme performance)
   - Optimized for read-heavy workloads
   - Memory-mapped architecture
   - ACID with optimistic concurrency
   - Lower write amplification than sled

3. **rocksdb** (For distributed deployments)
   - LSM-tree architecture
   - High write throughput
   - Tunable compaction strategies
   - Used by major distributed systems

**Data Model**:
```rust
// Tree structure in sled/redb
models:{id} → Model (bincode-serialized)
models:by_name:{name} → Set<ModelId>
models:by_tag:{tag} → Set<ModelId>
versions:{model_id}:{version} → ModelVersion
lineage:{id}:parents → Set<ArtifactId>
lineage:{id}:children → Set<ArtifactId>
metadata:{id} → Metadata (JSON)
```

#### Search Index

**Full-Text Search with Tantivy**:
- Lucene-inspired search engine in Rust
- Real-time indexing with commit control
- Faceted search and aggregations
- Fuzzy matching and phrase queries
- Custom scoring and relevance tuning

**Index Schema**:
```rust
pub fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("name", TEXT | STORED);
    schema_builder.add_text_field("description", TEXT);
    schema_builder.add_text_field("tags", TEXT);
    schema_builder.add_facet_field("category", INDEXED);
    schema_builder.add_date_field("created_at", INDEXED | STORED);
    schema_builder.add_json_field("metadata", TEXT | STORED);
    schema_builder.build()
}
```

**Search Capabilities**:
- Full-text search across name, description, metadata
- Faceted navigation (by category, tags, dates)
- Ranking by relevance, popularity, recency
- Query-time boosting and filtering
- Autocomplete and suggestions

#### Cache Layer

**Multi-Tier Caching Strategy**:

1. **In-Memory Cache** (`moka` or `mini-moka`)
   - LRU/LFU eviction policies
   - TTL-based expiration
   - Size-bounded cache
   - 10K-100K items typical

2. **Distributed Cache** (Optional, Redis/Valkey)
   - Shared cache across nodes
   - Pub/sub for invalidation
   - Persistence for warmup

**Cache Patterns**:
```rust
// Cache-aside pattern
async fn get_model(&self, id: &ModelId) -> Result<Model> {
    if let Some(model) = self.cache.get(id).await {
        return Ok(model);
    }

    let model = self.storage.get(id).await?;
    self.cache.insert(id, model.clone()).await;
    Ok(model)
}

// Write-through pattern
async fn update_model(&self, id: &ModelId, update: ModelUpdate) -> Result<Model> {
    let model = self.storage.update(id, update).await?;
    self.cache.insert(id, model.clone()).await;
    self.invalidate_related(id).await;
    Ok(model)
}
```

#### Blob Storage

**For Large Artifacts** (model weights, datasets):
- **Local**: Filesystem with content-addressable storage (CAS)
- **Remote**: S3-compatible object storage (via `rusoto_s3` or `aws-sdk-s3`)
- **Hybrid**: Local cache with remote backing store

**Storage Strategy**:
- Content hashing (SHA-256) for deduplication
- Multi-part upload for large files
- Presigned URLs for direct access
- Lifecycle policies for archival

#### Replication Log

**Append-Only Event Log**:
- Sequential event IDs for ordering
- Vector clocks for causality tracking
- Compaction for old events
- Point-in-time recovery

**Implementation Options**:
- Sled tree with append-only semantics
- External log (Kafka, NATS JetStream)
- Custom WAL implementation

---

### 4. Integration Layer

The Integration Layer connects LLM-Registry to ecosystem components.

#### Connector Architecture

**Base Connector Trait**:
```rust
#[async_trait]
pub trait Connector: Send + Sync {
    type Config: DeserializeOwned;
    type Event;

    async fn initialize(&mut self, config: Self::Config) -> Result<()>;
    async fn send_event(&self, event: Self::Event) -> Result<()>;
    async fn health_check(&self) -> Result<HealthStatus>;
    async fn shutdown(&mut self) -> Result<()>;
}
```

**Plugin System**:
- Dynamic loading with `libloading` (optional)
- Static compilation for security
- Configuration via TOML/YAML
- Lifecycle management (init, run, shutdown)

#### LLM-Memory-Graph Integration

**Graph Database Connector**:
- **Purpose**: Sync lineage and relationship data to graph database
- **Protocol**: GraphQL mutations or native graph DB protocol
- **Data Flow**: Registry → Connector → Memory Graph

**Lineage API**:
```rust
pub struct LineageConnector {
    client: GraphClient,
    batch_size: usize,
}

impl LineageConnector {
    async fn push_lineage(&self, node: LineageNode) -> Result<()> {
        // Convert registry lineage to graph nodes/edges
        let graph_node = self.convert_to_graph_node(node)?;
        self.client.create_node(graph_node).await?;
        Ok(())
    }

    async fn sync_relationships(&self, artifact_id: &ArtifactId) -> Result<()> {
        let relationships = self.registry.get_relationships(artifact_id).await?;
        for rel in relationships {
            self.client.create_edge(rel.source, rel.target, rel.type).await?;
        }
        Ok(())
    }
}
```

**Synchronization Strategy**:
- Push on write (real-time sync)
- Periodic batch sync (eventual consistency)
- Change stream subscription

#### LLM-Policy-Engine Integration

**Policy Evaluation Hooks**:
- **Purpose**: Enforce governance policies during artifact registration/updates
- **Protocol**: gRPC or HTTP callbacks
- **Data Flow**: Registry → Policy Engine → Decision

**Validation Callbacks**:
```rust
pub struct PolicyValidator {
    engine_client: PolicyEngineClient,
}

impl PolicyValidator {
    async fn validate(&self, artifact: &Artifact) -> Result<ValidationResult> {
        let request = PolicyEvaluationRequest {
            artifact_type: artifact.type_name(),
            metadata: artifact.metadata(),
            operation: Operation::Register,
            context: self.build_context(),
        };

        let response = self.engine_client.evaluate(request).await?;

        match response.decision {
            Decision::Allow => Ok(ValidationResult::Valid),
            Decision::Deny => Ok(ValidationResult::Invalid(response.reasons)),
            Decision::RequireApproval => Ok(ValidationResult::PendingApproval),
        }
    }
}
```

**Policy Types Enforced**:
- Metadata completeness requirements
- Naming conventions and standards
- License compatibility checks
- Security and compliance policies
- Quota and rate limits

#### LLM-Forge Integration

**SDK Metadata Sync**:
- **Purpose**: Bidirectional sync between Forge SDK and Registry
- **Protocol**: REST API + WebSocket for events
- **Data Flow**: Registry ↔ Forge SDK

**Sync Operations**:
```rust
pub struct ForgeConnector {
    api_client: RegistryApiClient,
    event_stream: WebSocketStream,
}

impl ForgeConnector {
    // Push metadata from Forge to Registry
    async fn publish_metadata(&self, metadata: ForgeMetadata) -> Result<ArtifactId> {
        let artifact = self.convert_to_artifact(metadata)?;
        self.api_client.register(artifact).await
    }

    // Pull updates from Registry to Forge
    async fn subscribe_updates(&self, artifact_id: ArtifactId) -> Result<()> {
        let mut stream = self.event_stream.subscribe(artifact_id).await?;
        while let Some(event) = stream.next().await {
            self.handle_update(event).await?;
        }
        Ok(())
    }

    // Bidirectional sync
    async fn sync_bidirectional(&self) -> Result<()> {
        // Merge strategy: last-write-wins with conflict resolution
        let local_changes = self.get_local_changes().await?;
        let remote_changes = self.api_client.get_changes_since(self.last_sync).await?;

        let merged = self.merge_changes(local_changes, remote_changes)?;
        self.apply_changes(merged).await?;
        Ok(())
    }
}
```

#### LLM-Governance-Dashboard Integration

**Analytics Feed**:
- **Purpose**: Provide real-time and historical analytics data
- **Protocol**: Server-Sent Events (SSE) or WebSocket
- **Data Flow**: Registry → Dashboard

**Admin APIs**:
```rust
// Analytics endpoints
GET /v1/analytics/models/count - Total models by category
GET /v1/analytics/usage/trends - Usage over time
GET /v1/analytics/lineage/complexity - Lineage graph metrics
GET /v1/analytics/compliance/status - Policy compliance stats

// Admin operations
POST /v1/admin/reindex - Trigger full reindex
POST /v1/admin/compact - Compact storage
GET /v1/admin/health - System health metrics
POST /v1/admin/backup - Create backup snapshot
```

**Metrics Exposed**:
- Total artifacts by type (models, datasets, prompts)
- Registration rate (artifacts/hour)
- Query latency percentiles (p50, p95, p99)
- Storage utilization (disk, cache hit rate)
- Replication lag (if distributed)
- Error rates by endpoint

#### External API Integrations

**HuggingFace Hub Sync**:
```rust
pub struct HuggingFaceImporter {
    client: HfApiClient,
}

impl HuggingFaceImporter {
    async fn import_model(&self, repo_id: &str) -> Result<ArtifactId> {
        let hf_model = self.client.get_model(repo_id).await?;
        let artifact = self.convert_hf_to_artifact(hf_model)?;
        self.registry.register(artifact).await
    }

    async fn sync_updates(&self) -> Result<()> {
        // Poll for updates or use webhooks
        let updates = self.client.get_recent_updates(self.last_sync).await?;
        for update in updates {
            self.sync_model(&update.repo_id).await?;
        }
        Ok(())
    }
}
```

**Event Publishers**:
- **Kafka**: High-throughput event streaming
- **NATS**: Lightweight pub/sub with JetStream persistence
- **Redis Streams**: Simple pub/sub with Redis infrastructure

---

## Rust Crate Recommendations

### Web Frameworks

#### axum (Recommended)
- **Version**: 0.7+
- **Rationale**:
  - Built on Tokio and Tower, excellent ecosystem integration
  - Type-safe extractors and handlers
  - Minimal boilerplate with ergonomic API
  - Great performance (handles 100K+ req/sec)
  - Easy testing with tower::ServiceExt
- **Use Cases**: REST API, WebSocket, primary HTTP interface
- **Example**:
  ```rust
  use axum::{Router, routing::get, extract::State, Json};

  async fn get_model(
      State(registry): State<Arc<RegistryService>>,
      Path(id): Path<ArtifactId>,
  ) -> Result<Json<Model>, ApiError> {
      let model = registry.get(&id).await?;
      Ok(Json(model))
  }

  let app = Router::new()
      .route("/v1/models/:id", get(get_model))
      .with_state(registry);
  ```

#### actix-web (Alternative)
- **Version**: 4.x
- **Rationale**:
  - Mature framework with actor-based architecture
  - Excellent performance benchmarks
  - Rich middleware ecosystem
  - Built-in extractors and guards
- **Trade-offs**: More opinionated, steeper learning curve
- **Use Cases**: When actor model fits application design

#### rocket (Not Recommended for Production)
- **Rationale**: Still pre-1.0, nightly Rust required historically
- **Use Cases**: Prototypes, learning

### Data Storage

#### sled (Primary Recommendation)
- **Version**: 0.34+
- **Rationale**:
  - Pure Rust embedded database
  - ACID transactions with serializable isolation
  - Zero-copy reads, crash recovery
  - Active development, good documentation
  - Production-ready (used in production by many)
- **Performance**: 40K writes/sec, 1M reads/sec (SSD)
- **Trade-offs**: Write amplification on large datasets
- **Configuration**:
  ```rust
  let db = sled::Config::new()
      .path("./data/registry.db")
      .cache_capacity(1024 * 1024 * 1024) // 1GB cache
      .mode(sled::Mode::HighThroughput)
      .open()?;
  ```

#### redb (Alternative)
- **Version**: 2.x
- **Rationale**:
  - Optimized for read-heavy workloads
  - Lower write amplification than sled
  - Memory-mapped I/O for fast reads
  - Simpler API surface
- **Trade-offs**: Younger project, smaller ecosystem
- **Use Cases**: Read-heavy registries with infrequent updates

#### rocksdb (Distributed Deployments)
- **Version**: 0.22+ (rust-rocksdb bindings)
- **Rationale**:
  - Battle-tested LSM-tree storage (Facebook, LinkedIn)
  - Excellent write throughput
  - Tunable compaction strategies
  - Column families for logical separation
- **Trade-offs**: C++ dependency, more complex configuration
- **Use Cases**: Multi-node clusters, high write throughput

#### sqlx (Structured Data Alternative)
- **Version**: 0.8+
- **Rationale**:
  - Compile-time checked SQL queries
  - Async/await native
  - Support for PostgreSQL, MySQL, SQLite
- **Trade-offs**: Requires SQL database, more operational overhead
- **Use Cases**: When SQL query capabilities are needed

### Serialization

#### serde (Core)
- **Version**: 1.x
- **Rationale**:
  - De facto standard for serialization in Rust
  - Zero-copy deserialization where possible
  - Extensive format support
  - Derive macros for boilerplate reduction

#### bincode (Binary Format)
- **Version**: 1.x
- **Rationale**:
  - Compact binary encoding
  - Very fast serialization/deserialization
  - Good for internal storage format
- **Performance**: ~10x faster than JSON
- **Trade-offs**: Not human-readable, format changes break compatibility

#### postcard (Embedded Binary)
- **Version**: 1.x
- **Rationale**:
  - Even more compact than bincode
  - No_std support for embedded use
  - Schema evolution support
- **Use Cases**: Embedded deployments, network protocols

#### serde_json (Human-Readable)
- **Version**: 1.x
- **Rationale**:
  - Human-readable, debuggable
  - Universal interoperability
  - Good for APIs and configuration
- **Trade-offs**: Larger size, slower than binary formats

#### rmp-serde (MessagePack)
- **Version**: 1.x
- **Rationale**:
  - Compact binary format like JSON
  - Good balance of size and interoperability
  - Language-agnostic
- **Use Cases**: API responses when bandwidth matters

### API Frameworks

#### async-graphql (GraphQL)
- **Version**: 7.x
- **Rationale**:
  - Full GraphQL spec implementation
  - Excellent Rust integration with procedural macros
  - DataLoader for N+1 query optimization
  - Subscription support
  - Built-in metrics and tracing
- **Example**:
  ```rust
  #[derive(SimpleObject)]
  struct Model {
      id: ID,
      name: String,
      version: String,
  }

  struct QueryRoot;

  #[Object]
  impl QueryRoot {
      async fn model(&self, ctx: &Context<'_>, id: ID) -> Result<Model> {
          ctx.data::<RegistryService>()?.get(&id).await
      }
  }
  ```

#### tonic (gRPC)
- **Version**: 0.12+
- **Rationale**:
  - Production-grade gRPC implementation
  - Generated code from protobuf with prost
  - Streaming support (client, server, bidirectional)
  - Excellent performance
  - Rich interceptor/middleware support
- **Use Cases**: Service-to-service communication, SDKs
- **Example**:
  ```protobuf
  service RegistryService {
    rpc GetModel(GetModelRequest) returns (Model);
    rpc StreamModels(StreamModelsRequest) returns (stream Model);
  }
  ```

#### poem (Alternative Web Framework)
- **Version**: 3.x
- **Rationale**:
  - OpenAPI integration built-in
  - Similar ergonomics to axum
  - Good for API-first development
- **Trade-offs**: Smaller ecosystem than axum

### Authentication & Authorization

#### jsonwebtoken (JWT)
- **Version**: 9.x
- **Rationale**:
  - Standard JWT implementation
  - Support for all common algorithms (HS256, RS256, ES256)
  - Token validation and claims extraction
- **Example**:
  ```rust
  let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))?;
  let token_data = decode::<Claims>(&token, &DecodingKey::from_secret(secret), &Validation::default())?;
  ```

#### oauth2 (OAuth2 Flows)
- **Version**: 4.x
- **Rationale**:
  - Complete OAuth2 client implementation
  - Support for all grant types
  - PKCE extension support
- **Use Cases**: Third-party authentication, SSO integration

#### openidconnect (OpenID Connect)
- **Version**: 3.x
- **Rationale**:
  - OpenID Connect on top of OAuth2
  - ID token validation
  - Discovery document support
- **Use Cases**: Enterprise SSO (Google, Azure AD, Okta)

### Search & Indexing

#### tantivy (Full-Text Search)
- **Version**: 0.22+
- **Rationale**:
  - Lucene-inspired search engine in pure Rust
  - Real-time indexing and search
  - Faceted search and aggregations
  - Excellent performance (millions of docs)
  - Customizable scoring and ranking
- **Example**:
  ```rust
  let mut index_writer = index.writer(50_000_000)?;
  index_writer.add_document(doc!(
      name_field => "llama-3-8b",
      description_field => "Large language model",
      tags_field => "llm,open-source"
  ))?;
  index_writer.commit()?;

  let query = QueryParser::for_index(&index, vec![name_field, description_field])
      .parse_query("language model")?;
  let searcher = index.reader()?.searcher();
  let results = searcher.search(&query, &TopDocs::with_limit(10))?;
  ```

#### meilisearch-sdk (External Search Engine)
- **Version**: 0.27+
- **Rationale**:
  - If embedding Meilisearch as a sidecar
  - Excellent search relevance out-of-box
  - Simple REST API
- **Trade-offs**: External process, more memory overhead

### Caching

#### moka (In-Memory Cache)
- **Version**: 0.12+
- **Rationale**:
  - Fast, concurrent cache implementation
  - LRU, LFU, and TTL eviction
  - Thread-safe with minimal contention
  - Size and time-based eviction
  - Async API support
- **Example**:
  ```rust
  let cache = Cache::builder()
      .max_capacity(10_000)
      .time_to_live(Duration::from_secs(300))
      .build();

  cache.insert(key, value);
  let value = cache.get(&key);
  ```

#### mini-moka (Lightweight Alternative)
- **Version**: 0.10+
- **Rationale**:
  - Smaller footprint than moka
  - Good for simple caching needs
  - Less feature-rich but faster

#### redis (Distributed Cache)
- **Version**: 0.27+ (redis-rs)
- **Rationale**:
  - Distributed caching across nodes
  - Pub/sub for cache invalidation
  - Persistence options
- **Use Cases**: Multi-node deployments

### Observability

#### tracing (Logging & Tracing)
- **Version**: 0.1.x
- **Rationale**:
  - Structured logging with spans
  - Async-aware tracing
  - Rich ecosystem (tracing-subscriber, tracing-opentelemetry)
- **Example**:
  ```rust
  #[tracing::instrument(skip(registry))]
  async fn get_model(id: ArtifactId, registry: &RegistryService) -> Result<Model> {
      tracing::info!(artifact_id = %id, "Fetching model");
      registry.get(&id).await
  }
  ```

#### opentelemetry (Distributed Tracing)
- **Version**: 0.24+
- **Rationale**:
  - Standard for distributed tracing
  - Integration with Jaeger, Zipkin, DataDog
  - Metrics and tracing in one
- **Use Cases**: Multi-service deployments

#### prometheus (Metrics)
- **Version**: 0.13+ (prometheus crate)
- **Rationale**:
  - Industry standard metrics format
  - Rich query language (PromQL)
  - Grafana integration
- **Metrics**:
  ```rust
  lazy_static! {
      static ref REGISTRY_REQUESTS: Counter = Counter::new("registry_requests_total", "Total requests").unwrap();
      static ref QUERY_DURATION: Histogram = Histogram::with_opts(
          HistogramOpts::new("query_duration_seconds", "Query latency")
      ).unwrap();
  }
  ```

### Additional Utilities

#### tokio (Async Runtime)
- **Version**: 1.x
- **Rationale**: De facto standard async runtime

#### tower (Middleware)
- **Version**: 0.5+
- **Rationale**: Composable middleware for services

#### thiserror (Error Handling)
- **Version**: 1.x
- **Rationale**: Ergonomic derive macros for Error types

#### anyhow (Error Propagation)
- **Version**: 1.x
- **Rationale**: Simplified error handling for applications

#### config (Configuration)
- **Version**: 0.14+
- **Rationale**: Hierarchical configuration from multiple sources

---

## Deployment Architectures

### 1. Embedded Mode

**Overview**: LLM-Registry runs as an in-process library embedded within another application.

```
┌─────────────────────────────────────────┐
│        Application Process              │
│  ┌──────────────────────────────────┐   │
│  │   Application Code               │   │
│  │   (LLM-Forge, Custom App)        │   │
│  └──────────┬───────────────────────┘   │
│             │                            │
│             │ Function Calls             │
│             │                            │
│  ┌──────────▼───────────────────────┐   │
│  │   Registry Library               │   │
│  │   - RegistryService              │   │
│  │   - Storage (sled/redb)          │   │
│  │   - Search Index (tantivy)       │   │
│  └──────────┬───────────────────────┘   │
│             │                            │
│             │ File I/O                   │
│             │                            │
│  ┌──────────▼───────────────────────┐   │
│  │   Local Storage                  │   │
│  │   ./data/registry.db             │   │
│  │   ./data/search.idx              │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

**Characteristics**:
- Zero network latency (in-process function calls)
- No separate deployment or infrastructure
- Single-process failure domain
- Shared memory and resources

**Configuration**:
```rust
use llm_registry::{Registry, RegistryConfig};

let config = RegistryConfig {
    storage_path: "./data/registry.db".into(),
    cache_size_mb: 256,
    enable_search: true,
    ..Default::default()
};

let registry = Registry::new(config).await?;

// Use directly
let model = registry.get_model(&model_id).await?;
registry.register_model(model_metadata).await?;
```

**Use Cases**:
- LLM-Forge SDK embedded metadata management
- Development and testing environments
- Edge deployments with limited resources
- Single-application metadata needs

**Limitations**:
- No network API (unless application provides one)
- No multi-process sharing
- Limited scalability (single process)
- No fault tolerance

**Dependencies**:
```toml
[dependencies]
llm-registry = { version = "0.1", default-features = false }
sled = "0.34"
tantivy = "0.22"
```

---

### 2. Standalone Mode

**Overview**: LLM-Registry runs as an independent microservice with REST/GraphQL/gRPC APIs.

```
                     ┌─────────────────────┐
                     │   Load Balancer     │
                     │   (nginx/haproxy)   │
                     └──────────┬──────────┘
                                │
         ┌──────────────────────┼──────────────────────┐
         │                      │                      │
┌────────▼────────┐    ┌────────▼────────┐    ┌───────▼─────────┐
│ Registry Node   │    │ Registry Node   │    │ Registry Node   │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ API Layer   │ │    │ │ API Layer   │ │    │ │ API Layer   │ │
│ │ (axum)      │ │    │ │ (axum)      │ │    │ │ (axum)      │ │
│ └──────┬──────┘ │    │ └──────┬──────┘ │    │ └──────┬──────┘ │
│        │        │    │        │        │    │        │        │
│ ┌──────▼──────┐ │    │ ┌──────▼──────┐ │    │ ┌──────▼──────┐ │
│ │ Service     │ │    │ │ Service     │ │    │ │ Service     │ │
│ │ Layer       │ │    │ │ Layer       │ │    │ │ Layer       │ │
│ └──────┬──────┘ │    │ └──────┬──────┘ │    │ └──────┬──────┘ │
│        │        │    │        │        │    │        │        │
│ ┌──────▼──────┐ │    │ ┌──────▼──────┐ │    │ ┌──────▼──────┐ │
│ │ Storage     │ │    │ │ Storage     │ │    │ │ Storage     │ │
│ │ (sled)      │ │    │ │ (sled)      │ │    │ │ (sled)      │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                      │                      │
         └──────────────────────┼──────────────────────┘
                                │
                    ┌───────────▼───────────┐
                    │  Shared Storage       │
                    │  (NFS/S3 for blobs)   │
                    └───────────────────────┘
```

**Characteristics**:
- Independent service lifecycle
- Horizontal scalability (stateless API layer)
- Network-based communication (REST/GraphQL/gRPC)
- Centralized metadata management

**Configuration**:
```toml
# config.toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[api]
enable_rest = true
enable_graphql = true
enable_grpc = true
cors_origins = ["*"]
rate_limit_per_min = 1000

[storage]
type = "sled"
path = "/var/lib/registry/data"
cache_size_mb = 1024

[search]
enabled = true
index_path = "/var/lib/registry/index"

[observability]
metrics_port = 9090
tracing_endpoint = "http://jaeger:14268/api/traces"
log_level = "info"
```

**Deployment**:
```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/llm-registry /usr/local/bin/
EXPOSE 8080 9090
CMD ["llm-registry", "--config", "/etc/registry/config.toml"]
```

```yaml
# docker-compose.yml
version: '3.8'
services:
  registry:
    image: llm-registry:latest
    ports:
      - "8080:8080"
      - "9090:9090"
    volumes:
      - registry-data:/var/lib/registry
      - ./config.toml:/etc/registry/config.toml
    environment:
      - RUST_LOG=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  registry-data:
```

**Use Cases**:
- Multi-application metadata sharing
- Microservices architecture
- Centralized governance and policy enforcement
- Production deployments with moderate scale

**Scalability**:
- Vertical: Increase CPU/RAM per instance
- Horizontal: Add more instances behind load balancer
- Read replicas: Add read-only instances

**High Availability**:
- Active-passive: Primary with standby failover
- Active-active: Multi-instance with shared storage
- Health checks and automatic failover

---

### 3. Distributed Mode

**Overview**: Multi-node cluster with data replication, partitioning, and consensus.

```
                          ┌───────────────────┐
                          │  API Gateway      │
                          │  (GraphQL/gRPC)   │
                          └────────┬──────────┘
                                   │
          ┌────────────────────────┼────────────────────────┐
          │                        │                        │
   ┌──────▼──────┐         ┌───────▼──────┐        ┌───────▼──────┐
   │  Registry   │         │  Registry    │        │  Registry    │
   │  Node 1     │◄────────┤  Node 2      │───────►│  Node 3      │
   │  (Leader)   │         │  (Follower)  │        │  (Follower)  │
   └──────┬──────┘         └───────┬──────┘        └───────┬──────┘
          │                        │                        │
          │     Raft Consensus     │                        │
          │◄──────────────────────►│◄──────────────────────►│
          │                        │                        │
   ┌──────▼──────┐         ┌───────▼──────┐        ┌───────▼──────┐
   │  Storage    │         │  Storage     │        │  Storage     │
   │  Partition  │         │  Partition   │        │  Partition   │
   │  A, B       │         │  B, C        │        │  C, A        │
   └─────────────┘         └──────────────┘        └──────────────┘

   Replication Factor: 2
   Consistency: Quorum (N/2 + 1)
```

**Characteristics**:
- Multi-node cluster with data partitioning
- Replication for fault tolerance
- Consensus protocol (Raft) for coordination
- Automatic failover and self-healing

**Architecture Patterns**:

**Leader-Follower Replication**:
- Single leader accepts writes
- Followers replicate from leader
- Read from any node (eventual consistency)
- Leader election on failure (Raft/Paxos)

**Multi-Leader Replication**:
- Multiple nodes accept writes
- Conflict resolution with CRDTs or LWW
- Higher write availability
- More complex conflict handling

**Leaderless Replication (Dynamo-style)**:
- Quorum reads/writes (W + R > N)
- Vector clocks for causality
- Anti-entropy (gossip, merkle trees)
- Tunable consistency (ONE, QUORUM, ALL)

**Implementation**:
```rust
use raft::{Config, Node, Storage};
use tokio::sync::mpsc;

pub struct DistributedRegistry {
    raft_node: Node<Storage>,
    local_storage: Arc<RegistryStorage>,
    replication_log: ReplicationLog,
}

impl DistributedRegistry {
    pub async fn new(cluster_config: ClusterConfig) -> Result<Self> {
        let raft_config = Config {
            id: cluster_config.node_id,
            election_tick: 10,
            heartbeat_tick: 3,
            max_size_per_msg: 1024 * 1024,
            max_inflight_msgs: 256,
            ..Default::default()
        };

        let raft_node = Node::new(&raft_config, storage, logger)?;

        Ok(Self {
            raft_node,
            local_storage: Arc::new(RegistryStorage::new()?),
            replication_log: ReplicationLog::new()?,
        })
    }

    pub async fn write(&self, key: ArtifactId, value: Artifact) -> Result<()> {
        // Propose write through Raft consensus
        let proposal = self.encode_proposal(Operation::Write(key, value))?;
        self.raft_node.propose(vec![0], proposal)?;

        // Wait for consensus
        self.wait_for_commit().await?;
        Ok(())
    }

    pub async fn read(&self, key: &ArtifactId) -> Result<Option<Artifact>> {
        // Read from local replica (eventual consistency)
        // Or perform quorum read for strong consistency
        if self.require_strong_consistency {
            self.quorum_read(key).await
        } else {
            self.local_storage.get(key).await
        }
    }
}
```

**Data Partitioning**:
```rust
// Consistent hashing for partition assignment
pub struct PartitionManager {
    ring: ConsistentHashRing,
    replication_factor: usize,
}

impl PartitionManager {
    pub fn assign_partition(&self, key: &ArtifactId) -> Vec<NodeId> {
        let hash = self.hash_key(key);
        self.ring.get_nodes(hash, self.replication_factor)
    }

    pub fn rebalance(&mut self, new_nodes: Vec<NodeId>) -> RebalancePlan {
        // Calculate partition movements
        // Minimize data transfer
        self.ring.add_nodes(new_nodes);
        self.generate_rebalance_plan()
    }
}
```

**Configuration**:
```toml
[cluster]
node_id = 1
peers = ["node2:8080", "node3:8080"]
replication_factor = 2

[consensus]
algorithm = "raft"
election_timeout_ms = 1000
heartbeat_interval_ms = 250

[partitioning]
strategy = "consistent_hash"
virtual_nodes = 150

[consistency]
read_consistency = "quorum"  # one, quorum, all
write_consistency = "quorum"
```

**Deployment** (Kubernetes):
```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: llm-registry
spec:
  serviceName: registry
  replicas: 3
  selector:
    matchLabels:
      app: registry
  template:
    metadata:
      labels:
        app: registry
    spec:
      containers:
      - name: registry
        image: llm-registry:distributed
        ports:
        - containerPort: 8080
          name: api
        - containerPort: 8090
          name: raft
        volumeMounts:
        - name: data
          mountPath: /var/lib/registry
        env:
        - name: NODE_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: CLUSTER_PEERS
          value: "registry-0.registry:8090,registry-1.registry:8090,registry-2.registry:8090"
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Gi
```

**Use Cases**:
- Large-scale deployments (100K+ artifacts)
- High availability requirements (99.99% uptime)
- Geographic distribution
- Regulatory data residency requirements

**Trade-offs**:
- Increased operational complexity
- Network latency between nodes
- CAP theorem considerations (CP or AP)
- More expensive infrastructure

**Crates for Distributed Systems**:
- **raft-rs**: Raft consensus implementation
- **hashring**: Consistent hashing
- **tokio**: Async runtime for node communication
- **tonic**: gRPC for inter-node RPC
- **dashmap**: Concurrent hashmap for shared state

---

## Integration Architecture

### LLM-Memory-Graph Integration

**Purpose**: Synchronize lineage and relationship data between Registry and graph database.

**Integration Points**:

1. **Lineage Graph Sync**
   - **Trigger**: Artifact registration, relationship creation
   - **Direction**: Registry → Memory Graph
   - **Protocol**: GraphQL mutations or native graph protocol (Cypher, Gremlin)

2. **Relationship Queries**
   - **Trigger**: Lineage visualization, impact analysis
   - **Direction**: Memory Graph → Registry (metadata enrichment)
   - **Protocol**: GraphQL queries

**Data Model Mapping**:
```rust
// Registry lineage node
pub struct LineageNode {
    pub artifact_id: ArtifactId,
    pub artifact_type: ArtifactType,
    pub parents: Vec<ArtifactId>,
    pub children: Vec<ArtifactId>,
    pub metadata: HashMap<String, Value>,
}

// Graph database representation (Cypher)
// CREATE (a:Artifact {id: $id, type: $type, metadata: $metadata})
// CREATE (a)-[:DERIVED_FROM]->(parent)
// CREATE (child)-[:DERIVED_FROM]->(a)
```

**Synchronization Strategy**:

**Push-based (Real-time)**:
```rust
pub struct MemoryGraphConnector {
    graph_client: GraphQLClient,
    batch_queue: Arc<Mutex<Vec<LineageEvent>>>,
}

impl MemoryGraphConnector {
    pub async fn on_artifact_registered(&self, artifact: &Artifact) -> Result<()> {
        let mutation = r#"
            mutation CreateArtifactNode($input: ArtifactInput!) {
                createArtifact(input: $input) {
                    id
                }
            }
        "#;

        let variables = json!({
            "input": {
                "id": artifact.id,
                "type": artifact.artifact_type,
                "metadata": artifact.metadata,
            }
        });

        self.graph_client.execute(mutation, variables).await?;
        Ok(())
    }

    pub async fn on_relationship_created(&self, rel: &Relationship) -> Result<()> {
        let mutation = r#"
            mutation CreateRelationship($from: ID!, $to: ID!, $type: RelationType!) {
                createRelationship(from: $from, to: $to, type: $type) {
                    id
                }
            }
        "#;

        self.graph_client.execute(mutation, json!({
            "from": rel.source,
            "to": rel.target,
            "type": rel.relationship_type,
        })).await?;
        Ok(())
    }
}
```

**Pull-based (Periodic Sync)**:
```rust
pub async fn sync_lineage_batch(&self) -> Result<()> {
    let since = self.get_last_sync_timestamp().await?;
    let changes = self.registry.get_lineage_changes_since(since).await?;

    for change in changes.batch(100) {
        self.push_to_graph(change).await?;
    }

    self.update_sync_timestamp().await?;
    Ok(())
}
```

**Conflict Resolution**:
- Registry is source of truth for artifact metadata
- Graph is source of truth for complex relationship queries
- Bidirectional sync with last-write-wins for conflicts

---

### LLM-Policy-Engine Integration

**Purpose**: Enforce governance policies during artifact lifecycle operations.

**Integration Points**:

1. **Pre-Registration Validation**
   - **Trigger**: Before artifact registration
   - **Decision**: Allow, Deny, RequireApproval
   - **Protocol**: gRPC or HTTP POST

2. **Post-Update Validation**
   - **Trigger**: After metadata update
   - **Decision**: Audit, Notify, Rollback

3. **Continuous Compliance Monitoring**
   - **Trigger**: Periodic scan
   - **Decision**: Flag non-compliant artifacts

**Policy Evaluation Flow**:
```
Artifact Registration Request
         │
         ▼
┌─────────────────────┐
│ Registry Service    │
│ (Pre-validation)    │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐      Policy Evaluation Request
│ Policy Engine       │◄──────────────────────────────────
│ Connector           │     {artifact, operation, context}
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ Policy Engine       │
│ - Load policies     │
│ - Evaluate rules    │
│ - Return decision   │
└─────────┬───────────┘
          │
          ▼         Decision: {allow, deny, reasons}
┌─────────────────────┐
│ Registry Service    │
│ - Allow: Proceed    │
│ - Deny: Return error│
│ - Approve: Queue    │
└─────────────────────┘
```

**Implementation**:
```rust
pub struct PolicyEngineConnector {
    client: PolicyEngineClient,
    cache: PolicyCache,
}

impl PolicyEngineConnector {
    pub async fn validate_artifact(&self, artifact: &Artifact, op: Operation) -> Result<ValidationResult> {
        // Build evaluation context
        let context = EvaluationContext {
            operation: op,
            artifact_type: artifact.artifact_type,
            user: self.get_current_user()?,
            timestamp: Utc::now(),
            metadata: artifact.metadata.clone(),
        };

        // Check cache for policy decisions (if deterministic)
        if let Some(cached) = self.cache.get(&artifact.id, &op) {
            return Ok(cached);
        }

        // Call policy engine
        let request = PolicyEvaluationRequest {
            artifact: artifact.clone(),
            context,
        };

        let response = self.client.evaluate(request).await?;

        // Cache decision
        self.cache.insert(&artifact.id, &op, response.decision.clone());

        match response.decision {
            Decision::Allow => Ok(ValidationResult::Valid),
            Decision::Deny => Ok(ValidationResult::Invalid(response.reasons)),
            Decision::RequireApproval => {
                self.create_approval_request(artifact, response.reasons).await?;
                Ok(ValidationResult::PendingApproval)
            }
        }
    }
}

// Integration in Registry Service
pub struct RegistryService {
    storage: Arc<Storage>,
    policy_validator: Arc<PolicyEngineConnector>,
}

impl RegistryService {
    pub async fn register(&self, artifact: Artifact) -> Result<ArtifactId> {
        // Pre-validation
        let validation = self.policy_validator.validate_artifact(&artifact, Operation::Register).await?;

        match validation {
            ValidationResult::Valid => {
                let id = self.storage.insert(artifact).await?;
                Ok(id)
            }
            ValidationResult::Invalid(reasons) => {
                Err(RegistryError::PolicyViolation(reasons))
            }
            ValidationResult::PendingApproval => {
                Err(RegistryError::ApprovalRequired)
            }
        }
    }
}
```

**Policy Examples**:
```yaml
# Metadata completeness policy
- name: require-license
  type: validation
  condition: artifact.metadata.license != null
  message: "License field is required for all models"

# Naming convention policy
- name: naming-convention
  type: validation
  condition: artifact.name matches "^[a-z0-9-]+$"
  message: "Names must be lowercase alphanumeric with hyphens"

# Quota policy
- name: user-quota
  type: authorization
  condition: user.artifact_count < user.quota_limit
  message: "User has reached artifact quota limit"
```

---

### LLM-Forge Integration

**Purpose**: Bidirectional synchronization between Forge SDK and Registry.

**Integration Patterns**:

1. **Metadata Publishing** (Forge → Registry)
   - SDK publishes model metadata to Registry
   - Registry validates and stores metadata

2. **Metadata Discovery** (Registry → Forge)
   - SDK queries Registry for available models
   - SDK downloads metadata for local use

3. **Bidirectional Sync** (Forge ↔ Registry)
   - Changes in either system propagate to the other
   - Conflict resolution with version vectors

**Sync Architecture**:
```
┌─────────────────────┐                    ┌─────────────────────┐
│   LLM-Forge SDK     │                    │   LLM-Registry      │
│                     │                    │                     │
│  ┌──────────────┐   │                    │  ┌──────────────┐   │
│  │ Local        │   │   Publish API      │  │ Validation   │   │
│  │ Metadata     │───┼───────────────────►│  │ Service      │   │
│  │ Cache        │   │   POST /v1/models  │  └──────┬───────┘   │
│  └──────────────┘   │                    │         │           │
│         ▲           │                    │         ▼           │
│         │           │                    │  ┌──────────────┐   │
│         │           │   Query API        │  │ Storage      │   │
│         │           │◄───────────────────┤  │              │   │
│         │           │   GET /v1/models   │  └──────────────┘   │
│         │           │                    │         │           │
│  ┌──────┴───────┐   │                    │         │           │
│  │ Sync         │   │   WebSocket        │  ┌──────▼───────┐   │
│  │ Manager      │◄──┼────────────────────┤  │ Event        │   │
│  │              │   │   (Change Events)  │  │ Publisher    │   │
│  └──────────────┘   │                    │  └──────────────┘   │
└─────────────────────┘                    └─────────────────────┘
```

**Implementation**:

**Forge SDK Client**:
```rust
pub struct RegistryClient {
    http_client: reqwest::Client,
    base_url: String,
    ws_connection: Option<WebSocketStream>,
}

impl RegistryClient {
    pub async fn publish_model(&self, model: ModelMetadata) -> Result<ModelId> {
        let response = self.http_client
            .post(format!("{}/v1/models", self.base_url))
            .json(&model)
            .send()
            .await?;

        let registered: RegisteredModel = response.json().await?;
        Ok(registered.id)
    }

    pub async fn query_models(&self, filter: ModelFilter) -> Result<Vec<ModelMetadata>> {
        let response = self.http_client
            .get(format!("{}/v1/models", self.base_url))
            .query(&filter)
            .send()
            .await?;

        response.json().await
    }

    pub async fn subscribe_changes(&mut self, model_id: ModelId) -> Result<ChangeStream> {
        let ws_url = format!("ws://{}/v1/events", self.base_url.replace("http://", ""));
        let (ws_stream, _) = connect_async(ws_url).await?;

        // Send subscription message
        ws_stream.send(Message::Text(json!({
            "type": "subscribe",
            "model_id": model_id,
        }).to_string())).await?;

        self.ws_connection = Some(ws_stream);
        Ok(ChangeStream::new(ws_stream))
    }
}
```

**Bidirectional Sync Manager**:
```rust
pub struct SyncManager {
    registry_client: RegistryClient,
    local_store: LocalMetadataStore,
    conflict_resolver: ConflictResolver,
}

impl SyncManager {
    pub async fn sync_bidirectional(&self) -> Result<SyncReport> {
        // 1. Get local changes since last sync
        let local_changes = self.local_store.get_changes_since(self.last_sync_time).await?;

        // 2. Get remote changes since last sync
        let remote_changes = self.registry_client.get_changes_since(self.last_sync_time).await?;

        // 3. Detect conflicts
        let conflicts = self.detect_conflicts(&local_changes, &remote_changes)?;

        // 4. Resolve conflicts
        let resolved = self.conflict_resolver.resolve(conflicts).await?;

        // 5. Apply local changes to remote
        for change in local_changes {
            self.registry_client.apply_change(change).await?;
        }

        // 6. Apply remote changes to local
        for change in remote_changes {
            self.local_store.apply_change(change).await?;
        }

        // 7. Apply resolved conflicts
        for resolution in resolved {
            self.apply_resolution(resolution).await?;
        }

        self.update_sync_time().await?;
        Ok(SyncReport::new())
    }

    fn detect_conflicts(&self, local: &[Change], remote: &[Change]) -> Result<Vec<Conflict>> {
        let mut conflicts = Vec::new();

        for local_change in local {
            for remote_change in remote {
                if local_change.artifact_id == remote_change.artifact_id {
                    // Both modified same artifact
                    if !self.is_ancestor(local_change, remote_change) {
                        conflicts.push(Conflict {
                            artifact_id: local_change.artifact_id,
                            local_version: local_change.version,
                            remote_version: remote_change.version,
                            local_change: local_change.clone(),
                            remote_change: remote_change.clone(),
                        });
                    }
                }
            }
        }

        Ok(conflicts)
    }
}

pub struct ConflictResolver {
    strategy: ConflictStrategy,
}

pub enum ConflictStrategy {
    LastWriteWins,
    ManualResolution,
    MergeBoth,
    LocalWins,
    RemoteWins,
}

impl ConflictResolver {
    pub async fn resolve(&self, conflicts: Vec<Conflict>) -> Result<Vec<Resolution>> {
        match self.strategy {
            ConflictStrategy::LastWriteWins => {
                conflicts.into_iter().map(|c| {
                    if c.local_change.timestamp > c.remote_change.timestamp {
                        Resolution::UseLocal(c)
                    } else {
                        Resolution::UseRemote(c)
                    }
                }).collect()
            }
            ConflictStrategy::ManualResolution => {
                // Prompt user for each conflict
                self.prompt_user_resolution(conflicts).await
            }
            ConflictStrategy::MergeBoth => {
                self.merge_conflicts(conflicts).await
            }
            _ => unimplemented!(),
        }
    }
}
```

**Event-Driven Updates**:
```rust
// Registry publishes change events
pub async fn on_artifact_updated(&self, artifact: &Artifact) -> Result<()> {
    let event = ChangeEvent {
        event_type: EventType::ArtifactUpdated,
        artifact_id: artifact.id,
        timestamp: Utc::now(),
        data: serde_json::to_value(artifact)?,
    };

    self.event_publisher.publish(event).await?;
    Ok(())
}

// Forge SDK subscribes to events
pub async fn handle_change_event(&self, event: ChangeEvent) -> Result<()> {
    match event.event_type {
        EventType::ArtifactUpdated => {
            let artifact: Artifact = serde_json::from_value(event.data)?;
            self.local_store.update(artifact).await?;
        }
        EventType::ArtifactDeleted => {
            self.local_store.delete(&event.artifact_id).await?;
        }
        _ => {}
    }
    Ok(())
}
```

---

### LLM-Governance-Dashboard Integration

**Purpose**: Provide analytics, monitoring, and administrative capabilities.

**Integration Points**:

1. **Analytics Feed** (Registry → Dashboard)
   - Real-time metrics stream
   - Historical analytics queries
   - Aggregated statistics

2. **Admin APIs** (Dashboard → Registry)
   - System configuration
   - Bulk operations
   - Maintenance tasks

**Analytics Architecture**:
```
┌─────────────────────────────────────────────────────────┐
│                  LLM-Registry                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │ Metrics Collector                               │    │
│  │ - Request counters                              │    │
│  │ - Latency histograms                            │    │
│  │ - Resource utilization                          │    │
│  └─────────────┬───────────────────────────────────┘    │
│                │                                         │
│                ▼                                         │
│  ┌─────────────────────────────────────────────────┐    │
│  │ Analytics Service                               │    │
│  │ - Aggregation (by time, category, user)        │    │
│  │ - Trend analysis                                │    │
│  │ - Anomaly detection                             │    │
│  └─────────────┬───────────────────────────────────┘    │
└────────────────┼───────────────────────────────────────-┘
                 │
                 │ SSE/WebSocket
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│          Governance Dashboard                           │
│  ┌─────────────────────────────────────────────────┐    │
│  │ Analytics UI                                    │    │
│  │ - Live metrics dashboards                       │    │
│  │ - Historical trends                             │    │
│  │ - Compliance reports                            │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

**Metrics API**:
```rust
// GET /v1/analytics/summary
#[derive(Serialize)]
pub struct AnalyticsSummary {
    pub total_artifacts: u64,
    pub artifacts_by_type: HashMap<ArtifactType, u64>,
    pub registrations_last_24h: u64,
    pub active_users: u64,
    pub storage_usage_mb: u64,
}

// GET /v1/analytics/trends
#[derive(Serialize)]
pub struct TrendData {
    pub time_series: Vec<TimePoint>,
    pub metric: MetricType,
}

pub struct TimePoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

// Implementation
pub async fn get_analytics_summary(
    State(analytics): State<Arc<AnalyticsService>>,
) -> Result<Json<AnalyticsSummary>, ApiError> {
    let summary = analytics.compute_summary().await?;
    Ok(Json(summary))
}

pub async fn stream_metrics(
    State(analytics): State<Arc<AnalyticsService>>,
) -> Sse<impl Stream<Item = Event>> {
    let stream = analytics.metrics_stream()
        .map(|metric| Event::default().json_data(metric).unwrap());

    Sse::new(stream)
}
```

**Admin APIs**:
```rust
// POST /v1/admin/reindex - Rebuild search index
pub async fn reindex_all(
    State(registry): State<Arc<RegistryService>>,
) -> Result<Json<ReindexStatus>, ApiError> {
    let status = registry.reindex_all_artifacts().await?;
    Ok(Json(status))
}

// POST /v1/admin/compact - Compact storage
pub async fn compact_storage(
    State(storage): State<Arc<Storage>>,
) -> Result<Json<CompactionStats>, ApiError> {
    let stats = storage.compact().await?;
    Ok(Json(stats))
}

// GET /v1/admin/health - System health
pub async fn health_check(
    State(registry): State<Arc<RegistryService>>,
) -> Result<Json<HealthStatus>, ApiError> {
    let health = HealthStatus {
        storage: registry.storage.health_check().await?,
        search: registry.search.health_check().await?,
        cache: registry.cache.health_check().await?,
        uptime: registry.uptime(),
    };
    Ok(Json(health))
}

// POST /v1/admin/backup - Create backup
pub async fn create_backup(
    State(storage): State<Arc<Storage>>,
    Json(request): Json<BackupRequest>,
) -> Result<Json<BackupInfo>, ApiError> {
    let backup_id = storage.create_backup(request.destination).await?;
    Ok(Json(BackupInfo { id: backup_id }))
}
```

---

## Data Flow Architecture

### Write Path (Ingestion → Storage)

**Flow Diagram**:
```
Client Request
    │
    ▼
┌───────────────────┐
│ API Layer         │
│ - Authentication  │
│ - Validation      │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ Service Layer     │
│ 1. Schema valid.  │
│ 2. Policy check   │
│ 3. Enrichment     │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ Transaction Coord │
│ - Begin txn       │
└────────┬──────────┘
         │
         ├─────────────────────┬─────────────────────┐
         ▼                     ▼                     ▼
┌────────────────┐    ┌────────────────┐    ┌────────────────┐
│ Primary Store  │    │ Search Index   │    │ Cache          │
│ - Write KV     │    │ - Index doc    │    │ - Invalidate   │
└────────┬───────┘    └────────┬───────┘    └────────┬───────┘
         │                     │                     │
         └─────────────────────┴─────────────────────┘
                            │
                            ▼
                 ┌───────────────────┐
                 │ Transaction Coord │
                 │ - Commit txn      │
                 └────────┬──────────┘
                          │
                          ▼
                 ┌───────────────────┐
                 │ Event Publisher   │
                 │ - Notify listeners│
                 └────────┬──────────┘
                          │
                          ▼
                  ┌──────────────────┐
                  │ Response to      │
                  │ Client           │
                  └──────────────────┘
```

**Implementation**:
```rust
pub async fn register_artifact(&self, artifact: Artifact) -> Result<ArtifactId> {
    // 1. Validate schema
    artifact.validate_schema()?;

    // 2. Policy check
    let validation = self.policy_validator.validate(&artifact, Operation::Register).await?;
    if !validation.is_valid() {
        return Err(RegistryError::PolicyViolation(validation.errors));
    }

    // 3. Enrich metadata
    let enriched = self.enrich_metadata(artifact).await?;

    // 4. Begin transaction
    let txn = self.storage.begin_transaction().await?;

    // 5. Write to storage (within transaction)
    let id = self.generate_id();
    txn.insert(&id, &enriched).await?;

    // 6. Update search index (transactional)
    txn.index_artifact(&id, &enriched).await?;

    // 7. Invalidate cache
    self.cache.invalidate_related(&id).await;

    // 8. Commit transaction (2PC if distributed)
    txn.commit().await?;

    // 9. Publish event (after commit)
    self.event_publisher.publish(Event::ArtifactRegistered {
        id,
        artifact: enriched,
        timestamp: Utc::now(),
    }).await?;

    // 10. Update metrics
    ARTIFACTS_REGISTERED.inc();

    Ok(id)
}

async fn enrich_metadata(&self, mut artifact: Artifact) -> Result<Artifact> {
    // Add server-side metadata
    artifact.metadata.insert("registered_at".to_string(), Utc::now().to_rfc3339().into());
    artifact.metadata.insert("registered_by".to_string(), self.get_current_user()?.into());

    // Compute derived fields
    if artifact.metadata.contains_key("model_size") {
        let size_category = self.categorize_size(artifact.metadata["model_size"].as_u64().unwrap());
        artifact.metadata.insert("size_category".to_string(), size_category.into());
    }

    Ok(artifact)
}
```

**Performance Optimizations**:
- Batch writes when possible (bulk import)
- Async indexing for non-critical paths
- Write-ahead logging (WAL) for durability
- Pipelining for distributed writes

**Latency Budget**:
```
Total:     50ms (p99)
├─ Auth:    2ms
├─ Valid:   5ms
├─ Policy:  10ms
├─ Write:   20ms
├─ Index:   10ms
└─ Event:   3ms
```

---

### Read Path (Query → Retrieval)

**Flow Diagram**:
```
Client Query
    │
    ▼
┌───────────────────┐
│ API Layer         │
│ - Parse query     │
│ - Authenticate    │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ Service Layer     │
│ - Query planning  │
└────────┬──────────┘
         │
         ▼
    ┌────────┐
    │ Cache? │───Yes──► Return from Cache
    └───┬────┘
        │ No
        ▼
  ┌──────────┐
  │ Index or │
  │ Storage? │
  └───┬──┬───┘
      │  │
  Full│  │Simple
  text│  │lookup
      │  │
      ▼  ▼
  ┌─────────┐  ┌─────────┐
  │ Search  │  │ Storage │
  │ Index   │  │ KV Get  │
  └────┬────┘  └────┬────┘
       │            │
       └──────┬─────┘
              ▼
      ┌──────────────┐
      │ Hydrate full │
      │ artifact     │
      └──────┬───────┘
             │
             ▼
      ┌──────────────┐
      │ Transform    │
      │ (projection) │
      └──────┬───────┘
             │
             ▼
      ┌──────────────┐
      │ Populate     │
      │ cache        │
      └──────┬───────┘
             │
             ▼
      ┌──────────────┐
      │ Return to    │
      │ client       │
      └──────────────┘
```

**Implementation**:
```rust
pub async fn get_artifact(&self, id: &ArtifactId) -> Result<Option<Artifact>> {
    // 1. Check cache
    if let Some(cached) = self.cache.get(id).await {
        CACHE_HITS.inc();
        return Ok(Some(cached));
    }
    CACHE_MISSES.inc();

    // 2. Retrieve from storage
    let artifact = self.storage.get(id).await?;

    // 3. Populate cache for next time
    if let Some(ref artifact) = artifact {
        self.cache.insert(id, artifact.clone()).await;
    }

    Ok(artifact)
}

pub async fn search_artifacts(&self, query: SearchQuery) -> Result<SearchResults> {
    // 1. Build search query
    let tantivy_query = self.build_tantivy_query(&query)?;

    // 2. Execute search
    let searcher = self.search_index.reader()?.searcher();
    let top_docs = searcher.search(&tantivy_query, &TopDocs::with_limit(query.limit))?;

    // 3. Retrieve full artifacts
    let mut artifacts = Vec::new();
    for (_score, doc_address) in top_docs {
        let doc = searcher.doc(doc_address)?;
        let id = self.extract_id(&doc)?;

        // Use batch get for efficiency
        if let Some(artifact) = self.get_artifact(&id).await? {
            artifacts.push(artifact);
        }
    }

    Ok(SearchResults {
        artifacts,
        total: top_docs.len(),
        query: query.clone(),
    })
}

fn build_tantivy_query(&self, query: &SearchQuery) -> Result<Box<dyn Query>> {
    let mut subqueries: Vec<Box<dyn Query>> = Vec::new();

    // Full-text search
    if let Some(ref text) = query.text {
        let parser = QueryParser::for_index(&self.index, vec![self.name_field, self.desc_field]);
        subqueries.push(Box::new(parser.parse_query(text)?));
    }

    // Filters
    if let Some(ref category) = query.category {
        subqueries.push(Box::new(TermQuery::new(
            Term::from_field_text(self.category_field, category),
            IndexRecordOption::Basic,
        )));
    }

    // Combine with BooleanQuery
    Ok(Box::new(BooleanQuery::new(subqueries)))
}
```

**Caching Strategy**:
```rust
pub struct MultiTierCache {
    l1_cache: Arc<MiniMoka<ArtifactId, Artifact>>,  // Hot data, 1K items
    l2_cache: Arc<Moka<ArtifactId, Artifact>>,      // Warm data, 10K items
}

impl MultiTierCache {
    pub async fn get(&self, id: &ArtifactId) -> Option<Artifact> {
        // Try L1 (hot cache)
        if let Some(artifact) = self.l1_cache.get(id) {
            return Some(artifact);
        }

        // Try L2 (warm cache)
        if let Some(artifact) = self.l2_cache.get(id).await {
            // Promote to L1
            self.l1_cache.insert(*id, artifact.clone());
            return Some(artifact);
        }

        None
    }

    pub async fn insert(&self, id: &ArtifactId, artifact: Artifact) {
        // Insert to both tiers
        self.l1_cache.insert(*id, artifact.clone());
        self.l2_cache.insert(*id, artifact).await;
    }
}
```

**Latency Budget**:
```
Cache hit:     1ms  (p99)
Cache miss:    20ms (p99)
Search query:  30ms (p99)
```

---

### Replication Path (Change Detection → Sync)

**Flow Diagram**:
```
Source Node (Write)
    │
    ▼
┌───────────────────┐
│ Write to local    │
│ storage + WAL     │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ Append to         │
│ replication log   │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ Replicate to      │
│ followers         │
│ (async)           │
└────────┬──────────┘
         │
         ├─────────────────┬─────────────────┐
         ▼                 ▼                 ▼
    ┌─────────┐       ┌─────────┐       ┌─────────┐
    │ Node 2  │       │ Node 3  │       │ Node N  │
    └────┬────┘       └────┬────┘       └────┬────┘
         │                 │                 │
         ▼                 ▼                 ▼
    ┌─────────┐       ┌─────────┐       ┌─────────┐
    │ Apply   │       │ Apply   │       │ Apply   │
    │ change  │       │ change  │       │ change  │
    └────┬────┘       └────┬────┘       └────┬────┘
         │                 │                 │
         └─────────────────┴─────────────────┘
                          │
                          ▼
                  ┌───────────────┐
                  │ Send ACK      │
                  │ to leader     │
                  └───────────────┘
```

**Implementation**:
```rust
pub struct ReplicationService {
    local_storage: Arc<Storage>,
    replication_log: ReplicationLog,
    followers: Vec<FollowerConnection>,
}

impl ReplicationService {
    pub async fn replicate_write(&self, change: Change) -> Result<()> {
        // 1. Write to local WAL
        self.replication_log.append(change.clone()).await?;

        // 2. Send to followers (async, don't block write)
        let futures = self.followers.iter().map(|follower| {
            let change = change.clone();
            async move {
                follower.send_change(change).await
            }
        });

        // 3. Wait for quorum (if strong consistency)
        let results = futures::future::join_all(futures).await;
        let acks = results.iter().filter(|r| r.is_ok()).count();

        if acks >= self.quorum_size() {
            Ok(())
        } else {
            Err(ReplicationError::QuorumNotReached)
        }
    }

    pub async fn apply_change(&self, change: Change) -> Result<()> {
        // Follower applies replicated change
        match change.operation {
            Operation::Insert => {
                self.local_storage.insert(&change.key, &change.value).await?;
            }
            Operation::Update => {
                self.local_storage.update(&change.key, &change.value).await?;
            }
            Operation::Delete => {
                self.local_storage.delete(&change.key).await?;
            }
        }

        // Update replication watermark
        self.update_watermark(change.sequence_number).await?;

        Ok(())
    }
}
```

**Conflict Resolution**:
```rust
pub struct ConflictResolver {
    strategy: ResolutionStrategy,
}

pub enum ResolutionStrategy {
    LastWriteWins,
    VectorClock,
    CRDT,
    Manual,
}

impl ConflictResolver {
    pub fn resolve(&self, local: &Change, remote: &Change) -> Resolution {
        match self.strategy {
            ResolutionStrategy::LastWriteWins => {
                if remote.timestamp > local.timestamp {
                    Resolution::AcceptRemote
                } else {
                    Resolution::KeepLocal
                }
            }
            ResolutionStrategy::VectorClock => {
                match self.compare_vector_clocks(&local.vector_clock, &remote.vector_clock) {
                    Ordering::Less => Resolution::AcceptRemote,
                    Ordering::Greater => Resolution::KeepLocal,
                    Ordering::Equal => Resolution::Merge,
                }
            }
            ResolutionStrategy::CRDT => {
                // Use CRDT merge semantics
                Resolution::Merge
            }
            ResolutionStrategy::Manual => {
                Resolution::RequireManual(Conflict {
                    local: local.clone(),
                    remote: remote.clone(),
                })
            }
        }
    }
}
```

**Consistency Models**:

1. **Eventual Consistency**
   - Asynchronous replication
   - No quorum required
   - Fastest writes, potential conflicts

2. **Strong Consistency**
   - Quorum writes (W + R > N)
   - Linearizable reads
   - Higher latency

3. **Causal Consistency**
   - Preserve causality with vector clocks
   - No global ordering required
   - Balance of performance and consistency

**Replication Lag Monitoring**:
```rust
pub async fn measure_replication_lag(&self) -> HashMap<NodeId, Duration> {
    let mut lags = HashMap::new();
    let local_watermark = self.get_local_watermark().await;

    for follower in &self.followers {
        let follower_watermark = follower.get_watermark().await;
        let lag = local_watermark - follower_watermark;
        lags.insert(follower.id, Duration::from_millis(lag as u64));
    }

    lags
}
```

---

## Summary

This architecture provides:

1. **Layered Design**: Clear separation of concerns (API, Service, Data, Integration)
2. **Multiple Deployment Modes**: Embedded, Standalone, Distributed
3. **Rust-First Approach**: Leveraging best-in-class Rust crates for each component
4. **Ecosystem Integration**: Seamless connections to Memory Graph, Policy Engine, Forge, and Dashboard
5. **Production-Ready**: Observability, health checks, metrics, distributed tracing
6. **Scalable**: From single-process to multi-node clusters
7. **Flexible**: Support for multiple protocols (REST, GraphQL, gRPC, WebSocket)

**Next Steps**:
- Implement proof-of-concept for each layer
- Benchmark storage backends (sled vs redb vs rocksdb)
- Design API schemas (OpenAPI, GraphQL SDL, protobuf)
- Develop integration adapters for ecosystem components
- Create deployment manifests (Docker, Kubernetes)
