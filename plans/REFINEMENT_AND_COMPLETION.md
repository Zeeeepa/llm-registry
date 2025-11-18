# REFINEMENT AND COMPLETION

## REFINEMENT

### 1. API Design Refinement

#### REST vs GraphQL Trade-offs

**REST API Decision (Primary Interface)**

For LLM-Registry, REST is recommended as the primary API interface with the following rationale:

**Advantages for LLM-Registry:**
- **Predictable resource patterns**: LLM artifacts map naturally to REST resources (`/models`, `/datasets`, `/prompts`, `/evaluations`)
- **HTTP caching**: Metadata queries benefit from standard HTTP caching mechanisms (ETags, Cache-Control)
- **Simpler client integration**: Most ML/AI tools have built-in HTTP clients
- **Better monitoring**: Standard HTTP status codes simplify observability
- **Lower complexity**: Easier to implement and maintain for CRUD operations
- **Bandwidth efficiency**: REST endpoints can be optimized per use case

**GraphQL as Optional Layer (Beta Phase)**

GraphQL should be considered for Beta phase to address specific use cases:
- **Complex relationship queries**: When querying model lineage with dependencies
- **Custom field selection**: Reducing payload size for clients with specific needs
- **Batch queries**: Fetching multiple related resources in one request
- **Real-time subscriptions**: For registry change notifications

**Decision Matrix:**

| Use Case | REST | GraphQL | Rationale |
|----------|------|---------|-----------|
| Register new model | Primary | - | Simple POST operation |
| Query by ID | Primary | Optional | Direct resource access |
| List with filters | Primary | Optional | Query params sufficient |
| Complex lineage queries | - | Primary | Nested relationships |
| Batch operations | REST batch endpoint | Primary | Both viable |
| Real-time updates | Server-Sent Events | Subscriptions | Both viable |

#### Versioning Strategy

**Recommended: URL Path Versioning**

```
/api/v1/models
/api/v2/models
```

**Rationale:**
- **Explicit and visible**: Version is immediately clear in URLs
- **Client-friendly**: Easy to route to different implementations
- **Cache-friendly**: Different versions cached separately
- **Developer experience**: Clear migration path for clients
- **Backward compatibility**: Old versions can coexist indefinitely

**Version Lifecycle:**
- **v1 (MVP)**: Core CRUD operations, basic metadata
- **v2 (Beta)**: Advanced queries, lineage tracking, multi-tenancy
- **v3 (Future)**: Enhanced analytics, AI-driven recommendations

**Deprecation Policy:**
- Minimum 12-month support for deprecated versions
- Clear deprecation headers: `Sunset: Sat, 31 Dec 2026 23:59:59 GMT`
- Migration guides and compatibility tools provided

#### Pagination, Filtering, and Sorting

**Pagination Strategy: Cursor-based with Offset Fallback**

```http
GET /api/v1/models?limit=50&cursor=eyJpZCI6MTIzNDU2fQ==
```

**Response:**
```json
{
  "data": [...],
  "pagination": {
    "next_cursor": "eyJpZCI6MTIzNTA2fQ==",
    "has_more": true,
    "total": 1523
  }
}
```

**Benefits:**
- **Consistency**: Cursor ensures stable pagination even with concurrent writes
- **Performance**: Index-based cursor navigation is O(1)
- **Scalability**: Works well with distributed systems

**Offset Pagination (Simple Queries):**
```http
GET /api/v1/models?page=2&per_page=20
```

Use for: Small result sets, user-facing pagination, when total count is needed

**Filtering Best Practices:**

```http
# Simple filters
GET /api/v1/models?framework=pytorch&status=active

# Range filters
GET /api/v1/models?created_after=2025-01-01&size_mb_lt=1000

# Array filters
GET /api/v1/models?tags=nlp,transformers&architecture=encoder-decoder

# Search
GET /api/v1/models?q=gpt&search_fields=name,description
```

**Filtering Implementation:**
- **Indexed fields**: All filter fields should have database indexes
- **Filter validation**: Whitelist allowed filter fields to prevent injection
- **Complex queries**: Use POST with query DSL for advanced filtering
- **Performance limits**: Maximum 10 filter conditions per request

**Sorting Strategy:**

```http
GET /api/v1/models?sort=-created_at,name
```

- Prefix `-` for descending, no prefix for ascending
- Multiple sort fields supported (comma-separated)
- Default sort: `-created_at` (newest first)
- Only allow sorting on indexed fields

#### Batch Operations

**Batch Read:**
```http
POST /api/v1/models/batch/get
{
  "ids": ["model-123", "model-456", "model-789"]
}
```

**Batch Create:**
```http
POST /api/v1/models/batch
{
  "models": [
    { "name": "model-1", ... },
    { "name": "model-2", ... }
  ]
}
```

**Response:**
```json
{
  "results": [
    { "id": "model-123", "status": "success" },
    { "id": "model-456", "status": "error", "error": "Validation failed" }
  ],
  "summary": {
    "total": 2,
    "success": 1,
    "failed": 1
  }
}
```

**Batch Guidelines:**
- **Max batch size**: 100 items per request
- **Partial success**: Continue processing on individual failures
- **Atomic option**: `?atomic=true` for all-or-nothing behavior
- **Rate limiting**: Batch operations count as N requests for rate limiting
- **Async processing**: Large batches (>50) processed asynchronously with status endpoint

---

### 2. Authentication & Authorization Refinement

#### Multi-tenant Isolation Mechanisms

**Tenant Model:**
- **Organization-level tenancy**: Each organization is a tenant
- **Workspace support**: Organizations can have multiple workspaces
- **Resource ownership**: All resources belong to organization + workspace

**Database Isolation:**
```sql
-- Logical isolation with Row-Level Security (RLS)
CREATE POLICY tenant_isolation ON models
  USING (organization_id = current_setting('app.current_org_id')::uuid);

-- Indexes include tenant ID
CREATE INDEX idx_models_org_created ON models(organization_id, created_at);
```

**Isolation Strategies:**

| Approach | MVP | Beta | v1 | Rationale |
|----------|-----|------|----|-----------|
| Logical (RLS) | ✓ | ✓ | ✓ | Cost-effective, simpler ops |
| Schema per tenant | - | - | Optional | For very large tenants |
| Database per tenant | - | - | Optional | For compliance isolation |

**Tenant Context Propagation:**
```javascript
// Middleware extracts tenant from JWT
app.use(async (req, res, next) => {
  const orgId = req.user.organization_id;
  await db.query('SET app.current_org_id = $1', [orgId]);
  next();
});
```

#### Role-Based Access Control (RBAC) Model

**Role Hierarchy:**

```
Super Admin (Platform)
  └─ Organization Owner
      ├─ Organization Admin
      │   ├─ Workspace Admin
      │   │   ├─ Developer
      │   │   ├─ Data Scientist
      │   │   └─ Viewer
      │   └─ Auditor
      └─ Billing Admin
```

**Permission Model:**

```json
{
  "roles": {
    "org_owner": {
      "permissions": ["*"],
      "inherit": []
    },
    "org_admin": {
      "permissions": [
        "models:*",
        "datasets:*",
        "workspaces:*",
        "users:read",
        "users:invite"
      ],
      "inherit": []
    },
    "developer": {
      "permissions": [
        "models:create",
        "models:read",
        "models:update",
        "datasets:create",
        "datasets:read",
        "evaluations:*"
      ],
      "inherit": ["viewer"]
    },
    "viewer": {
      "permissions": [
        "models:read",
        "datasets:read",
        "evaluations:read"
      ],
      "inherit": []
    }
  }
}
```

**Permission Granularity:**
- **Resource-level**: `models:read`, `datasets:create`
- **Action-level**: `read`, `create`, `update`, `delete`, `share`
- **Attribute-based**: Filter by tags, ownership, sensitivity level

**Implementation:**
```javascript
// Permission check middleware
async function requirePermission(resource, action) {
  return async (req, res, next) => {
    const hasPermission = await authService.checkPermission(
      req.user,
      resource,
      action,
      { organizationId: req.org.id }
    );

    if (!hasPermission) {
      return res.status(403).json({ error: 'Forbidden' });
    }
    next();
  };
}

// Usage
app.post('/api/v1/models',
  authenticate,
  requirePermission('models', 'create'),
  createModel
);
```

#### API Key vs JWT vs OAuth2

**Authentication Matrix:**

| Client Type | Authentication Method | Use Case |
|-------------|----------------------|----------|
| Web Application | JWT (access + refresh) | Interactive user sessions |
| Mobile App | JWT + OAuth2 PKCE | Mobile clients |
| CLI Tool | API Key or Device Flow | Developer tools |
| Service-to-Service | Service Account JWT | Internal microservices |
| Third-party Integration | OAuth2 Authorization Code | External apps accessing user data |
| CI/CD Pipeline | API Key (scoped) | Automated deployments |
| SDK | API Key or OAuth2 | Programmatic access |

**API Key Design (MVP):**

```json
{
  "key_id": "llmreg_sk_live_abc123...",
  "name": "Production Training Pipeline",
  "scopes": ["models:write", "datasets:read"],
  "organization_id": "org_xyz",
  "workspace_id": "ws_123",
  "rate_limit": "1000/hour",
  "expires_at": "2026-12-31T23:59:59Z",
  "created_at": "2025-11-17T00:00:00Z",
  "last_used_at": "2025-11-17T12:34:56Z"
}
```

**API Key Best Practices:**
- **Prefix-based**: `llmreg_sk_live_` for easy identification
- **Scoped permissions**: Principle of least privilege
- **Rotation support**: Overlap period for key rotation
- **Usage tracking**: Monitor for anomalies
- **Rate limiting**: Per-key limits

**JWT Design (Beta):**

```json
{
  "access_token": {
    "sub": "user_123",
    "org_id": "org_xyz",
    "workspace_id": "ws_123",
    "roles": ["developer"],
    "permissions": ["models:read", "models:create"],
    "exp": 1700000000,
    "iat": 1699996400
  },
  "refresh_token": {
    "sub": "user_123",
    "jti": "refresh_token_id",
    "exp": 1732532400
  }
}
```

**Token Lifetimes:**
- **Access Token**: 15 minutes (short-lived)
- **Refresh Token**: 30 days (stored in httpOnly cookie)
- **API Key**: User-defined (max 1 year)

**OAuth2 Flow (v1):**

```
Authorization Code Flow (Web Apps):
1. Client redirects to /oauth/authorize
2. User authenticates and consents
3. Redirect to client with authorization code
4. Client exchanges code for tokens
5. Use access token for API calls
6. Refresh when expired

Device Flow (CLI Tools):
1. CLI requests device code
2. User visits URL and enters code
3. CLI polls for token
4. Token granted after approval
```

#### Service-to-Service Authentication

**Service Account Model:**

```json
{
  "service_account": {
    "id": "sa_training_pipeline_prod",
    "name": "Training Pipeline - Production",
    "organization_id": "org_xyz",
    "type": "service_account",
    "credentials": {
      "type": "jwt",
      "private_key_id": "key_123",
      "token_uri": "https://registry.example.com/oauth/token"
    },
    "permissions": [
      "models:create",
      "models:read",
      "datasets:read"
    ]
  }
}
```

**mTLS for Service Mesh (v1):**
- Certificate-based authentication for microservices
- Automatic certificate rotation
- Zero-trust network model

**Implementation Example:**
```javascript
// Service account authentication
async function authenticateService(req, res, next) {
  const token = req.headers.authorization?.replace('Bearer ', '');

  try {
    const payload = jwt.verify(token, publicKey, {
      issuer: 'llm-registry',
      audience: 'api.llm-registry.com'
    });

    if (payload.type !== 'service_account') {
      throw new Error('Invalid token type');
    }

    req.serviceAccount = payload;
    next();
  } catch (error) {
    res.status(401).json({ error: 'Invalid service token' });
  }
}
```

---

### 3. Storage Optimization

#### Indexing Strategies

**Primary Indexes (MVP):**

```sql
-- Models table
CREATE TABLE models (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id UUID NOT NULL,
  workspace_id UUID NOT NULL,
  name TEXT NOT NULL,
  framework TEXT NOT NULL,
  version TEXT NOT NULL,
  status TEXT NOT NULL,
  tags TEXT[] DEFAULT '{}',
  metadata JSONB DEFAULT '{}',
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ
);

-- Essential indexes
CREATE INDEX idx_models_org_workspace ON models(organization_id, workspace_id)
  WHERE deleted_at IS NULL;

CREATE INDEX idx_models_created_at ON models(created_at DESC)
  WHERE deleted_at IS NULL;

CREATE INDEX idx_models_name ON models(organization_id, name)
  WHERE deleted_at IS NULL;

CREATE INDEX idx_models_framework ON models(framework)
  WHERE deleted_at IS NULL;

CREATE INDEX idx_models_status ON models(organization_id, status)
  WHERE deleted_at IS NULL;

-- JSONB indexes for metadata queries
CREATE INDEX idx_models_metadata_gin ON models USING GIN(metadata);

-- Array index for tags
CREATE INDEX idx_models_tags_gin ON models USING GIN(tags);

-- Full-text search
CREATE INDEX idx_models_fts ON models USING GIN(
  to_tsvector('english', name || ' ' || COALESCE(metadata->>'description', ''))
);
```

**Index Optimization Guidelines:**
- **Partial indexes**: Use `WHERE deleted_at IS NULL` for soft deletes
- **Multi-column indexes**: Leftmost column should be most selective
- **Index size monitoring**: Track index bloat, rebuild when >20% bloat
- **Query-driven**: Create indexes based on actual query patterns
- **EXPLAIN ANALYZE**: Always verify index usage

**Advanced Indexing (Beta):**

```sql
-- Composite index for common filter combinations
CREATE INDEX idx_models_composite ON models(
  organization_id,
  framework,
  status,
  created_at DESC
) WHERE deleted_at IS NULL;

-- Hash index for exact equality lookups
CREATE INDEX idx_models_id_hash ON models USING HASH(id);

-- Expression index for computed values
CREATE INDEX idx_models_size_category ON models(
  CASE
    WHEN (metadata->>'size_mb')::numeric < 100 THEN 'small'
    WHEN (metadata->>'size_mb')::numeric < 1000 THEN 'medium'
    ELSE 'large'
  END
) WHERE deleted_at IS NULL;
```

#### Caching Layers

**Three-tier Caching Strategy:**

```
┌─────────────────────────────────────────┐
│         Client-side Cache               │
│  (Browser, SDK in-memory cache)         │
│  TTL: 5 minutes                         │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│      Application Cache (Redis)          │
│  - Query results                        │
│  - Aggregations                         │
│  - Session data                         │
│  TTL: 15 minutes                        │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│      Database Query Cache               │
│  - PostgreSQL shared_buffers            │
│  - Query plan cache                     │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│          Database Storage               │
└─────────────────────────────────────────┘
```

**Redis Cache Strategy (Beta):**

```javascript
// Cache key pattern
const cacheKey = `models:${orgId}:${queryHash}`;

// Read-through cache
async function getModels(orgId, filters) {
  const key = buildCacheKey('models', orgId, filters);

  // Try cache first
  const cached = await redis.get(key);
  if (cached) {
    return JSON.parse(cached);
  }

  // Query database
  const results = await db.query(buildQuery(filters));

  // Cache with TTL
  await redis.setex(key, 900, JSON.stringify(results)); // 15 min

  return results;
}

// Cache invalidation on writes
async function createModel(modelData) {
  const model = await db.insert(modelData);

  // Invalidate relevant caches
  await redis.del(`models:${model.organization_id}:*`);

  return model;
}
```

**Cache Patterns:**

| Data Type | Strategy | TTL | Invalidation |
|-----------|----------|-----|--------------|
| Model metadata | Read-through | 15 min | On write |
| Search results | Cache-aside | 5 min | Time-based |
| Aggregations | Write-through | 1 hour | On relevant writes |
| User sessions | Write-through | 30 min | On logout |
| Rate limit counters | Write-through | 1 hour | Rolling window |

**HTTP Caching Headers:**

```http
# Immutable resources (specific version)
Cache-Control: public, max-age=31536000, immutable
ETag: "abc123..."

# Mutable resources (latest version)
Cache-Control: public, max-age=300, must-revalidate
ETag: "def456..."
Last-Modified: Mon, 17 Nov 2025 12:00:00 GMT

# Private user data
Cache-Control: private, max-age=0, no-cache
```

#### Compression Strategies

**Payload Compression:**

```javascript
// Automatic compression middleware
app.use(compression({
  filter: (req, res) => {
    // Compress responses > 1KB
    return compression.filter(req, res);
  },
  threshold: 1024,
  level: 6 // Balance between speed and compression ratio
}));
```

**Metadata Compression (Database):**

```sql
-- JSONB already compressed, but for large text fields
CREATE TABLE model_artifacts (
  id UUID PRIMARY KEY,
  model_id UUID REFERENCES models(id),
  content BYTEA, -- Compressed with pg_compress
  compression TEXT DEFAULT 'gzip',
  original_size BIGINT,
  compressed_size BIGINT
);

-- Compression function
CREATE OR REPLACE FUNCTION compress_content(text)
RETURNS BYTEA AS $$
  SELECT pg_compress($1::bytea, 'gzip')
$$ LANGUAGE SQL;

-- Decompression
CREATE OR REPLACE FUNCTION decompress_content(bytea)
RETURNS TEXT AS $$
  SELECT convert_from(pg_decompress($1, 'gzip'), 'UTF8')
$$ LANGUAGE SQL;
```

**Response Compression Trade-offs:**

| Compression | Ratio | CPU Cost | Use Case |
|-------------|-------|----------|----------|
| None | 1x | None | Small payloads (<1KB) |
| gzip (level 6) | 3-5x | Low | Most API responses |
| gzip (level 9) | 5-8x | Medium | Large static metadata |
| brotli (level 4) | 4-6x | Medium | Modern clients, static assets |

**Large Metadata Optimization:**

```json
{
  "model_id": "model_123",
  "metadata_summary": {
    "framework": "pytorch",
    "version": "2.0.1"
  },
  "metadata_full_url": "https://storage.example.com/metadata/model_123.json.gz",
  "metadata_size_bytes": 5242880,
  "metadata_compressed_size_bytes": 1048576
}
```

For metadata >1MB:
- Store in object storage (S3-compatible)
- Return URL in API response
- Compress with gzip
- Optional: Generate signed URLs for security

#### Partitioning Strategies

**Time-based Partitioning (v1):**

```sql
-- Partition models by creation year
CREATE TABLE models_2025 PARTITION OF models
  FOR VALUES FROM ('2025-01-01') TO ('2026-01-01');

CREATE TABLE models_2026 PARTITION OF models
  FOR VALUES FROM ('2026-01-01') TO ('2027-01-01');

-- Automated partition management
CREATE OR REPLACE FUNCTION create_yearly_partitions()
RETURNS void AS $$
DECLARE
  year INT;
BEGIN
  FOR year IN
    SELECT EXTRACT(YEAR FROM NOW())::INT + i
    FROM generate_series(0, 2) i
  LOOP
    EXECUTE format('
      CREATE TABLE IF NOT EXISTS models_%s PARTITION OF models
      FOR VALUES FROM (%L) TO (%L)',
      year,
      make_date(year, 1, 1),
      make_date(year + 1, 1, 1)
    );
  END LOOP;
END;
$$ LANGUAGE plpgsql;
```

**List Partitioning by Organization (Large Deployments):**

```sql
-- For very large tenants requiring isolation
CREATE TABLE models_org_large1 PARTITION OF models
  FOR VALUES IN ('org_abc123');

-- Default partition for smaller orgs
CREATE TABLE models_org_default PARTITION OF models
  DEFAULT;
```

**Partitioning Decision Matrix:**

| Scale | Strategy | When to Apply |
|-------|----------|---------------|
| <10M rows | No partitioning | MVP, Beta |
| 10M-100M rows | Time-based (yearly) | v1 |
| >100M rows | Time + tenant hybrid | Large deployments |
| Compliance | Tenant partitioning | Data residency requirements |

**Benefits:**
- **Query performance**: Partition pruning reduces scan size
- **Maintenance**: Easier to archive/drop old partitions
- **Parallelism**: Queries can run parallel on partitions
- **Backup/restore**: Partition-level operations

---

### 4. Replication Refinement

#### Consistency Models

**Recommended: Tunable Consistency (Beta)**

```
┌─────────────────────────────────────────┐
│         Write Request                    │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Primary Node                     │
│  - Synchronous replication to 1 replica │
│  - Async to remaining replicas          │
└─────────────────────────────────────────┘
         ↓ sync      ↓ async
┌──────────────┐  ┌──────────────┐
│   Replica 1  │  │   Replica 2  │
│   (Sync)     │  │   (Async)    │
└──────────────┘  └──────────────┘
```

**Consistency Levels:**

| Level | Description | Latency | Use Case |
|-------|-------------|---------|----------|
| Strong | All replicas acknowledge | High (100-500ms) | Critical metadata writes |
| Quorum | Majority (N/2+1) acknowledge | Medium (50-100ms) | Default for writes |
| Eventual | Primary only, async to replicas | Low (5-10ms) | High-throughput ingestion |
| Read-your-writes | Session consistency | Low-Medium | User-facing operations |

**Implementation:**

```javascript
// Write with consistency level
async function writeModel(modelData, options = {}) {
  const consistency = options.consistency || 'quorum';

  switch (consistency) {
    case 'strong':
      // Wait for all replicas
      return await db.write(modelData, {
        synchronous_commit: 'remote_apply'
      });

    case 'quorum':
      // Wait for majority
      return await db.write(modelData, {
        synchronous_commit: 'on'
      });

    case 'eventual':
      // Return immediately
      return await db.write(modelData, {
        synchronous_commit: 'local'
      });
  }
}

// Read preference
async function readModel(id, options = {}) {
  const preference = options.readPreference || 'primary-preferred';

  const node = await selectNode(preference);
  return await node.query('SELECT * FROM models WHERE id = $1', [id]);
}
```

**PostgreSQL Synchronous Replication:**

```sql
-- postgresql.conf
synchronous_commit = on
synchronous_standby_names = 'replica1'

-- This ensures writes wait for replica1 before acknowledging
```

#### Conflict Resolution Strategies

**Last-Write-Wins (LWW) with Vector Clocks (v1):**

```javascript
// Model version tracking
{
  "id": "model_123",
  "name": "my-model-v2",
  "version_vector": {
    "node1": 5,
    "node2": 3
  },
  "updated_at": "2025-11-17T12:00:00Z",
  "updated_by": "user_456"
}

// Conflict detection
function detectConflict(versionA, versionB) {
  const aGreater = Object.keys(versionA).some(
    node => versionA[node] > (versionB[node] || 0)
  );
  const bGreater = Object.keys(versionB).some(
    node => versionB[node] > (versionA[node] || 0)
  );

  return aGreater && bGreater; // Concurrent modifications
}

// Resolve by timestamp
function resolveConflict(recordA, recordB) {
  return recordA.updated_at > recordB.updated_at ? recordA : recordB;
}
```

**Conflict Resolution Strategies:**

| Resource Type | Strategy | Rationale |
|---------------|----------|-----------|
| Model metadata | Last-write-wins | Simple, works for append-mostly |
| Tags | Set union | Combine tags from both versions |
| Metrics | Application-specific | Depends on metric type |
| Permissions | Explicit merge | Security-critical |
| Audit logs | Preserve all | No conflicts, append-only |

**Operational Transform for Concurrent Edits (Advanced):**

```javascript
// For collaborative editing scenarios
function transform(op1, op2) {
  // Transform op1 against op2
  if (op1.field !== op2.field) {
    return op1; // No conflict
  }

  // Field-level merge
  if (op1.type === 'set' && op2.type === 'set') {
    return {
      ...op1,
      timestamp: Math.max(op1.timestamp, op2.timestamp)
    };
  }

  // Array operations
  if (op1.type === 'push' && op2.type === 'push') {
    return {
      type: 'push',
      field: op1.field,
      values: [...op1.values, ...op2.values]
    };
  }
}
```

#### Sync Protocols

**Hybrid Push-Pull Protocol (Beta):**

```
Primary → Replicas: Push (Real-time)
Replicas → Primary: Pull (Periodic reconciliation)

┌─────────────────────────────────────────┐
│           Write to Primary               │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│    WAL Streaming (Push)                  │
│    - PostgreSQL logical replication      │
│    - Low latency (<100ms)                │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Replica Nodes                    │
│    - Apply changes                       │
│    - Periodic pull for reconciliation    │
└─────────────────────────────────────────┘
                  ↑
┌─────────────────────────────────────────┐
│    Reconciliation Pull (every 5 min)     │
│    - Verify consistency                  │
│    - Repair drift                        │
└─────────────────────────────────────────┘
```

**PostgreSQL Logical Replication:**

```sql
-- On primary
CREATE PUBLICATION llm_registry_pub FOR ALL TABLES;

-- On replica
CREATE SUBSCRIPTION llm_registry_sub
  CONNECTION 'host=primary dbname=llmregistry'
  PUBLICATION llm_registry_pub;
```

**Benefits:**
- **Low latency**: WAL streaming is near real-time
- **Reliability**: Pull ensures eventual consistency
- **Flexibility**: Can filter which tables to replicate

**Alternative: Event-based Replication (v1):**

```javascript
// Publish changes to event stream
async function publishModelChange(event) {
  await kafka.publish('model-changes', {
    event_type: 'model.created',
    model_id: event.id,
    organization_id: event.org_id,
    timestamp: Date.now(),
    data: event
  });
}

// Replicas subscribe and apply
kafka.subscribe('model-changes', async (message) => {
  await replica.applyChange(message);
});
```

#### Network Partition Handling

**Split-brain Prevention (v1):**

```
┌─────────────────────────────────────────┐
│         Consensus Layer                  │
│    - Raft/Paxos for leader election     │
│    - Majority quorum required           │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│      Leader (Accepts Writes)             │
└─────────────────────────────────────────┘
         ↓                ↓
┌──────────────┐  ┌──────────────┐
│  Follower 1  │  │  Follower 2  │
│  (Read-only) │  │  (Read-only) │
└──────────────┘  └──────────────┘
```

**Partition Detection:**

```javascript
// Heartbeat mechanism
class ClusterMonitor {
  async checkHealth() {
    const nodes = await this.getClusterNodes();

    for (const node of nodes) {
      try {
        const response = await axios.get(
          `http://${node.host}/health`,
          { timeout: 1000 }
        );

        if (response.data.status !== 'ok') {
          await this.handleUnhealthyNode(node);
        }
      } catch (error) {
        await this.handlePartitionedNode(node);
      }
    }
  }

  async handlePartitionedNode(node) {
    // Mark node as unavailable
    await this.updateNodeStatus(node.id, 'partitioned');

    // Redirect traffic
    await this.removeFromLoadBalancer(node);

    // Alert operators
    await this.sendAlert({
      severity: 'high',
      message: `Node ${node.id} partitioned`
    });
  }
}
```

**Partition Strategies:**

| Scenario | Action | Recovery |
|----------|--------|----------|
| Minority partition | Reject writes, serve reads | Auto-rejoin when healed |
| Majority partition | Continue operations | Sync minority nodes |
| Equal split | Reject writes (no quorum) | Manual intervention |

**PostgreSQL Patroni for HA:**

```yaml
# patroni.yml
scope: llm-registry
namespace: /service/
name: node1

restapi:
  listen: 0.0.0.0:8008
  connect_address: node1:8008

etcd:
  hosts: etcd1:2379,etcd2:2379,etcd3:2379

bootstrap:
  dcs:
    ttl: 30
    loop_wait: 10
    retry_timeout: 10
    maximum_lag_on_failover: 1048576

postgresql:
  listen: 0.0.0.0:5432
  connect_address: node1:5432
  data_dir: /var/lib/postgresql/data
  parameters:
    max_connections: 100
    shared_buffers: 256MB
    wal_level: replica
    hot_standby: on
```

**Benefits:**
- **Automatic failover**: Promotes replica in <30 seconds
- **Split-brain prevention**: Uses etcd consensus
- **Health monitoring**: Continuous checks

---

## COMPLETION

### Phased Roadmap

## Phase 1: MVP (Minimum Viable Product)

**Timeline: 12-16 weeks**

### Feature Scope

#### Core Features

**1. Registration System**
- **Model Registration**: Register PyTorch, TensorFlow, ONNX models
  - Required fields: name, framework, version, organization_id
  - Optional: description, tags, hyperparameters
  - Size limit: 10GB per model
- **Dataset Registration**: Register training/validation datasets
  - Metadata: name, size, format, schema
  - References to storage locations (URLs)
- **Basic Metadata**: JSON structure with validation
  ```json
  {
    "name": "gpt-classifier-v1",
    "framework": "pytorch",
    "version": "1.0.0",
    "description": "Binary classifier for sentiment",
    "tags": ["nlp", "classification"],
    "hyperparameters": {
      "learning_rate": 0.001,
      "batch_size": 32
    },
    "metrics": {
      "accuracy": 0.94,
      "f1_score": 0.92
    }
  }
  ```

**2. Query System**
- **GET by ID**: Retrieve single resource
  - `/api/v1/models/{id}`
  - `/api/v1/datasets/{id}`
- **List with Basic Filters**:
  - Filter by: framework, status, tags
  - Sort by: created_at, updated_at, name
  - Pagination: Offset-based (page, per_page)
- **Search**: Simple text search on name and description
  - `/api/v1/models?q=classifier`

**3. REST API**
- **Endpoints**:
  - `POST /api/v1/models` - Create model
  - `GET /api/v1/models/{id}` - Get model
  - `GET /api/v1/models` - List models
  - `PUT /api/v1/models/{id}` - Update model
  - `DELETE /api/v1/models/{id}` - Soft delete model
  - Same pattern for `/datasets`
- **Response Format**: JSON
  ```json
  {
    "data": { ... },
    "meta": {
      "request_id": "req_abc123",
      "timestamp": "2025-11-17T12:00:00Z"
    }
  }
  ```
- **Error Handling**: Standard HTTP status codes
  ```json
  {
    "error": {
      "code": "validation_error",
      "message": "Invalid model framework",
      "details": {
        "framework": "Must be one of: pytorch, tensorflow, onnx"
      }
    }
  }
  ```

**4. Authentication**
- **API Key Based**: Simple bearer token
  - `Authorization: Bearer llmreg_sk_live_abc123...`
- **Organization Scoping**: All requests scoped to org
- **Basic Rate Limiting**: 1000 requests/hour per key

**5. Single-node Deployment**
- **Architecture**:
  ```
  ┌─────────────────┐
  │   Load Balancer │
  │   (nginx)       │
  └────────┬────────┘
           │
  ┌────────▼────────┐
  │   API Server    │
  │   (Node.js)     │
  └────────┬────────┘
           │
  ┌────────▼────────┐
  │   PostgreSQL    │
  │   (Single node) │
  └─────────────────┘
  ```
- **Database**: PostgreSQL 15+
- **Storage**: Local filesystem or S3-compatible
- **No replication**: Single point of failure acceptable for MVP

**6. Ecosystem Integration (1-2 Components)**

**Integration 1: MLflow**
```python
# MLflow plugin for LLM Registry
import mlflow
from llm_registry_client import LLMRegistry

class LLMRegistryPlugin(mlflow.tracking.MlflowClient):
    def __init__(self, registry_url, api_key):
        self.registry = LLMRegistry(registry_url, api_key)

    def log_model(self, run_id, model_path):
        # Extract metadata from MLflow run
        run = self.get_run(run_id)

        # Register in LLM Registry
        self.registry.register_model({
            "name": run.data.tags["model_name"],
            "framework": run.data.tags["framework"],
            "mlflow_run_id": run_id,
            "metrics": run.data.metrics,
            "params": run.data.params
        })
```

**Integration 2: Weights & Biases**
```python
# W&B callback
import wandb
from llm_registry_client import LLMRegistry

class LLMRegistryCallback(wandb.Callback):
    def __init__(self, api_key):
        self.registry = LLMRegistry(
            "https://api.llm-registry.com",
            api_key
        )

    def on_train_end(self, logs=None):
        # Register model after training
        self.registry.register_model({
            "name": wandb.run.name,
            "wandb_run_id": wandb.run.id,
            "metrics": wandb.run.summary,
            "config": wandb.config
        })
```

### Dependencies and Prerequisites

**Technical Prerequisites:**
- [ ] PostgreSQL 15+ database provisioned
- [ ] Domain and SSL certificates configured
- [ ] Object storage (S3/MinIO) available
- [ ] CI/CD pipeline set up
- [ ] Monitoring infrastructure (basic Prometheus + Grafana)

**Team Prerequisites:**
- [ ] 2-3 backend engineers
- [ ] 1 frontend engineer (for basic admin UI)
- [ ] 1 DevOps engineer
- [ ] Technical writer for documentation

**Third-party Dependencies:**
- [ ] PostgreSQL database
- [ ] Object storage provider
- [ ] SSL certificate provider
- [ ] Monitoring/logging service

### Validation Metrics and Success Criteria

**Functional Metrics:**
- [ ] 100% of core API endpoints functional
- [ ] All CRUD operations working for models and datasets
- [ ] API key authentication working
- [ ] 1-2 ecosystem integrations functional

**Performance Metrics:**
- [ ] API response time: <200ms (p95) for GET requests
- [ ] API response time: <500ms (p95) for POST/PUT requests
- [ ] Support 100 concurrent users
- [ ] Handle 10,000 models in registry

**Reliability Metrics:**
- [ ] 95% uptime during MVP phase
- [ ] Zero data loss incidents
- [ ] All critical errors logged and alerted

**User Validation:**
- [ ] 5 beta users successfully integrated
- [ ] Positive feedback from user interviews
- [ ] Documentation enables self-service onboarding

### Timeline

**Weeks 1-4: Foundation**
- Week 1-2: Database schema, API framework setup
- Week 3-4: Core CRUD operations, authentication

**Weeks 5-8: Feature Development**
- Week 5-6: Query system, filtering, search
- Week 7-8: First ecosystem integration (MLflow)

**Weeks 9-12: Integration & Testing**
- Week 9-10: Second ecosystem integration (W&B)
- Week 11: End-to-end testing
- Week 12: Beta user testing

**Weeks 13-16: Polish & Launch**
- Week 13-14: Bug fixes, documentation
- Week 15: Security audit
- Week 16: MVP launch

**Milestones:**
- [ ] Week 4: Core API functional
- [ ] Week 8: First integration complete
- [ ] Week 12: Beta testing begins
- [ ] Week 16: MVP release

### Scaling Considerations

**MVP Scale Targets:**
- **Users**: 10-50 organizations
- **Models**: 1,000-10,000 models
- **Traffic**: 100 requests/second peak
- **Storage**: 1TB total

**Known Limitations (to address in Beta):**
- Single point of failure (no replication)
- Basic query capabilities (no advanced lineage)
- No multi-tenant isolation
- Limited rate limiting
- No GraphQL API

---

## Phase 2: Beta

**Timeline: 16-20 weeks**

### Feature Scope

#### Advanced Features

**1. Advanced Queries**

**Lineage Tracking:**
```javascript
// Track model-to-model lineage
{
  "model_id": "model_123",
  "lineage": {
    "parent_models": ["model_100", "model_105"],
    "training_datasets": ["dataset_50", "dataset_51"],
    "derived_models": ["model_130", "model_131"]
  }
}

// Query API
GET /api/v1/models/{id}/lineage?depth=2
{
  "model": { ... },
  "lineage_graph": {
    "nodes": [
      { "id": "model_123", "type": "model", "name": "my-model" },
      { "id": "dataset_50", "type": "dataset", "name": "train-data" }
    ],
    "edges": [
      { "from": "dataset_50", "to": "model_123", "type": "trained_on" }
    ]
  }
}
```

**Dependency Tracking:**
```javascript
// Model dependencies
{
  "model_id": "model_123",
  "dependencies": {
    "libraries": [
      { "name": "pytorch", "version": "2.0.1" },
      { "name": "transformers", "version": "4.30.0" }
    ],
    "external_models": [
      { "name": "bert-base-uncased", "source": "huggingface" }
    ],
    "hardware_requirements": {
      "min_gpu_memory_gb": 16,
      "recommended_gpu": "A100"
    }
  }
}

// Query for compatibility
GET /api/v1/models?compatible_with=cuda:11.8,pytorch:2.0
```

**Advanced Tag Queries:**
```javascript
// Tag hierarchy and relationships
{
  "tags": {
    "domain": ["nlp", "computer-vision"],
    "task": ["classification", "sentiment-analysis"],
    "architecture": ["transformer", "encoder-only"],
    "status": ["production", "stable"]
  }
}

// Complex tag queries
GET /api/v1/models?tags=nlp+classification&tags_exclude=deprecated
GET /api/v1/models?tags_any=pytorch,tensorflow&tags_all=production,tested
```

**2. Multi-tenant Support**

**Tenant Isolation:**
```sql
-- Row-level security
CREATE POLICY tenant_isolation ON models
  USING (organization_id = current_setting('app.current_org_id')::uuid);

-- Tenant-specific quotas
{
  "organization_id": "org_123",
  "quotas": {
    "max_models": 1000,
    "max_storage_gb": 500,
    "max_api_calls_per_hour": 10000
  },
  "usage": {
    "models_count": 450,
    "storage_used_gb": 230,
    "api_calls_last_hour": 3200
  }
}
```

**Workspace Feature:**
```javascript
// Organizations can have multiple workspaces
{
  "organization_id": "org_123",
  "workspaces": [
    {
      "id": "ws_prod",
      "name": "Production",
      "models_count": 50
    },
    {
      "id": "ws_dev",
      "name": "Development",
      "models_count": 200
    }
  ]
}

// Access control per workspace
GET /api/v1/workspaces/ws_prod/models
```

**3. GraphQL API**

**Schema:**
```graphql
type Model {
  id: ID!
  name: String!
  framework: Framework!
  version: String!
  tags: [String!]!
  metadata: JSON
  lineage: Lineage
  createdAt: DateTime!
  updatedAt: DateTime!
}

type Lineage {
  parentModels: [Model!]!
  trainingDatasets: [Dataset!]!
  derivedModels: [Model!]!
}

type Query {
  model(id: ID!): Model
  models(
    filter: ModelFilter
    sort: ModelSort
    limit: Int
    cursor: String
  ): ModelConnection!

  # Complex lineage query
  modelLineage(id: ID!, depth: Int = 2): LineageGraph!
}

type Mutation {
  createModel(input: CreateModelInput!): Model!
  updateModel(id: ID!, input: UpdateModelInput!): Model!
  deleteModel(id: ID!): Boolean!
}

# Subscriptions for real-time updates
type Subscription {
  modelUpdated(organizationId: ID!): Model!
  modelCreated(organizationId: ID!): Model!
}
```

**Query Example:**
```graphql
query GetModelWithLineage {
  model(id: "model_123") {
    name
    framework
    lineage {
      parentModels {
        id
        name
        createdAt
      }
      trainingDatasets {
        id
        name
        size
      }
    }
    metadata
  }
}
```

**4. Integration with All 4 Ecosystem Components**

**Integration 3: HuggingFace Hub**
```python
# Sync models from HuggingFace to Registry
from huggingface_hub import HfApi
from llm_registry_client import LLMRegistry

class HFRegistrySync:
    def __init__(self, hf_token, registry_api_key):
        self.hf = HfApi(token=hf_token)
        self.registry = LLMRegistry(
            "https://api.llm-registry.com",
            registry_api_key
        )

    def sync_model(self, model_id):
        # Get model info from HF
        model_info = self.hf.model_info(model_id)

        # Register in LLM Registry
        self.registry.register_model({
            "name": model_id,
            "framework": model_info.pipeline_tag,
            "source": "huggingface",
            "external_id": model_id,
            "tags": model_info.tags,
            "downloads": model_info.downloads,
            "metadata": {
                "library_name": model_info.library_name,
                "model_type": model_info.modelId
            }
        })
```

**Integration 4: TensorBoard**
```python
# TensorBoard plugin
from tensorboard.plugins import base_plugin
from llm_registry_client import LLMRegistry

class LLMRegistryPlugin(base_plugin.TBPlugin):
    def __init__(self, context):
        self.registry = LLMRegistry(
            context.config.registry_url,
            context.config.api_key
        )

    def get_plugin_apps(self):
        return {
            '/register': self._serve_register_model,
            '/models': self._serve_models_list
        }

    def _serve_register_model(self, request):
        # Extract run metadata
        run_name = request.args.get('run')

        # Register in LLM Registry
        self.registry.register_model({
            "name": run_name,
            "framework": "tensorflow",
            "source": "tensorboard",
            "tensorboard_run": run_name
        })
```

**5. Replication (2-node Setup)**

**Architecture:**
```
┌─────────────────────────────────────────┐
│         Load Balancer                    │
│         (Read/Write splitting)           │
└────────┬─────────────────────┬──────────┘
         │                     │
    ┌────▼────┐          ┌────▼────┐
    │ Primary │◄─────────│ Replica │
    │ (Write) │  Sync    │ (Read)  │
    └─────────┘          └─────────┘
```

**Configuration:**
```yaml
# Patroni cluster config
scope: llm-registry
namespace: /service/
name: node1

bootstrap:
  dcs:
    synchronous_mode: true
    postgresql:
      parameters:
        synchronous_commit: on
        synchronous_standby_names: 'node2'

# Read/write splitting in application
database:
  primary:
    host: primary.llm-registry.internal
    port: 5432
  replicas:
    - host: replica1.llm-registry.internal
      port: 5432
```

**Read/Write Splitting:**
```javascript
class DatabaseRouter {
  async query(sql, params, options = {}) {
    const useReplica = options.readOnly &&
                       options.staleRead !== false;

    const connection = useReplica
      ? await this.getReplicaConnection()
      : await this.getPrimaryConnection();

    return await connection.query(sql, params);
  }
}

// Usage
// Reads go to replica
await db.query('SELECT * FROM models', [], { readOnly: true });

// Writes go to primary
await db.query('INSERT INTO models ...', params, { readOnly: false });
```

### Dependencies and Prerequisites

**Infrastructure:**
- [ ] Second database node provisioned
- [ ] Patroni cluster configured
- [ ] etcd cluster for consensus (3 nodes)
- [ ] Redis cluster for caching (2 nodes)
- [ ] Load balancer with health checks

**Team Expansion:**
- [ ] +1 backend engineer (GraphQL specialist)
- [ ] +1 frontend engineer
- [ ] +1 integration engineer
- [ ] Technical writer

**Integration Prerequisites:**
- [ ] Access to HuggingFace API
- [ ] TensorBoard plugin development environment
- [ ] Test accounts for all integrations

### Validation Metrics and Success Criteria

**Functional Metrics:**
- [ ] GraphQL API 100% feature parity with REST
- [ ] All 4 ecosystem integrations functional
- [ ] Multi-tenant isolation verified (security audit)
- [ ] Replication lag <100ms (p95)

**Performance Metrics:**
- [ ] API response time: <100ms (p95) for GET requests
- [ ] Complex lineage queries: <500ms (p95)
- [ ] Support 1,000 concurrent users
- [ ] Handle 100,000 models in registry
- [ ] Read throughput: 10,000 queries/second

**Reliability Metrics:**
- [ ] 99% uptime
- [ ] Automatic failover tested and working
- [ ] Zero data loss during replica failover
- [ ] Replication lag monitoring and alerting

**User Validation:**
- [ ] 50+ beta organizations
- [ ] 90% user satisfaction (survey)
- [ ] <10% API error rate
- [ ] All critical user feedback addressed

### Timeline

**Weeks 1-5: Multi-tenancy & Advanced Queries**
- Week 1-2: Multi-tenant database isolation, RBAC
- Week 3-4: Lineage tracking, dependency queries
- Week 5: Advanced tag queries and search

**Weeks 6-10: GraphQL & Integrations**
- Week 6-7: GraphQL API development
- Week 8-9: HuggingFace integration
- Week 10: TensorBoard integration

**Weeks 11-15: Replication & Scaling**
- Week 11-12: 2-node replication setup
- Week 13: Read/write splitting
- Week 14: Caching layer (Redis)
- Week 15: Performance optimization

**Weeks 16-20: Testing & Hardening**
- Week 16-17: Load testing, security audit
- Week 18: Beta user testing (expanded group)
- Week 19: Bug fixes, monitoring improvements
- Week 20: Beta release

**Milestones:**
- [ ] Week 5: Multi-tenancy complete
- [ ] Week 10: All integrations functional
- [ ] Week 15: Replication stable
- [ ] Week 20: Beta release

### Scaling Considerations

**Beta Scale Targets:**
- **Users**: 50-500 organizations
- **Models**: 10,000-100,000 models
- **Traffic**: 1,000 requests/second peak
- **Storage**: 10TB total
- **Concurrent users**: 1,000

**Scaling Strategies:**
- **Read replicas**: Add more read replicas as needed
- **Caching**: Redis cluster for hot data
- **Connection pooling**: PgBouncer for database connections
- **CDN**: CloudFront for static assets

---

## Phase 3: v1 (Production Ready)

**Timeline: 20-24 weeks**

### Feature Scope

#### Production Features

**1. Full Distributed Cluster Support**

**Multi-region Architecture:**
```
Region 1 (US-East)          Region 2 (EU-West)
┌──────────────────┐        ┌──────────────────┐
│   API Servers    │        │   API Servers    │
│   (Auto-scaling) │        │   (Auto-scaling) │
└────────┬─────────┘        └────────┬─────────┘
         │                           │
┌────────▼─────────┐        ┌────────▼─────────┐
│  Primary DB      │◄──────►│  Primary DB      │
│  (Read/Write)    │  Sync  │  (Read/Write)    │
└────────┬─────────┘        └────────┬─────────┘
         │                           │
    ┌────┴────┐                 ┌────┴────┐
    │ Replica │                 │ Replica │
    └─────────┘                 └─────────┘
```

**Kubernetes Deployment:**
```yaml
# API deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-registry-api
spec:
  replicas: 5
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 2
      maxUnavailable: 1
  template:
    spec:
      containers:
      - name: api
        image: llm-registry/api:v1.0.0
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5

---
# Horizontal Pod Autoscaler
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: llm-registry-api-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: llm-registry-api
  minReplicas: 5
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

**Database Clustering (Patroni + Citus):**
```yaml
# Distributed PostgreSQL with Citus
patroni:
  scope: llm-registry-cluster
  nodes:
    - name: coordinator-1
      host: coordinator-1.db.internal
      role: coordinator
    - name: worker-1
      host: worker-1.db.internal
      role: worker
    - name: worker-2
      host: worker-2.db.internal
      role: worker
    - name: worker-3
      host: worker-3.db.internal
      role: worker

citus:
  # Distribute models table across workers
  distributed_tables:
    - table: models
      distribution_column: organization_id
    - table: datasets
      distribution_column: organization_id
```

**2. Advanced RBAC and Audit Logging**

**Fine-grained Permissions:**
```javascript
// Attribute-based access control
{
  "user_id": "user_123",
  "roles": ["developer"],
  "permissions": [
    {
      "resource": "models",
      "actions": ["read", "create", "update"],
      "conditions": {
        "organization_id": "org_123",
        "workspace_id": "ws_prod",
        "tags": { "contains": "production" }
      }
    }
  ],
  "custom_policies": [
    {
      "name": "production-model-approval",
      "rule": "models.tags.includes('production') => require_approval(['org_admin'])"
    }
  ]
}
```

**Permission Evaluation:**
```javascript
class PermissionEvaluator {
  async canPerform(user, action, resource, context) {
    // Check role-based permissions
    const rolePermissions = await this.getRolePermissions(user.roles);

    // Check custom policies
    const policies = await this.getUserPolicies(user.id);

    // Evaluate conditions
    for (const permission of rolePermissions) {
      if (this.matchesConditions(permission.conditions, context)) {
        if (permission.actions.includes(action)) {
          return true;
        }
      }
    }

    // Evaluate custom policies
    for (const policy of policies) {
      const result = await this.evaluatePolicy(policy, resource, context);
      if (result === 'deny') {
        return false;
      }
    }

    return false;
  }
}
```

**Comprehensive Audit Logging:**
```javascript
// Audit log schema
{
  "id": "audit_abc123",
  "timestamp": "2025-11-17T12:00:00Z",
  "organization_id": "org_123",
  "user_id": "user_456",
  "action": "model.update",
  "resource": {
    "type": "model",
    "id": "model_123",
    "name": "my-model-v2"
  },
  "changes": {
    "before": { "version": "1.0.0", "status": "draft" },
    "after": { "version": "2.0.0", "status": "production" }
  },
  "metadata": {
    "ip_address": "203.0.113.1",
    "user_agent": "llm-registry-cli/1.0.0",
    "request_id": "req_xyz789"
  },
  "result": "success"
}

// Query audit logs
GET /api/v1/audit-logs?user_id=user_456&action=model.update&from=2025-11-01
```

**Audit Log Storage:**
```sql
-- Partitioned by month for efficient querying
CREATE TABLE audit_logs (
  id UUID PRIMARY KEY,
  timestamp TIMESTAMPTZ NOT NULL,
  organization_id UUID NOT NULL,
  user_id UUID,
  action TEXT NOT NULL,
  resource JSONB NOT NULL,
  changes JSONB,
  metadata JSONB,
  result TEXT NOT NULL
) PARTITION BY RANGE (timestamp);

-- Indexes
CREATE INDEX idx_audit_logs_org_time ON audit_logs(organization_id, timestamp DESC);
CREATE INDEX idx_audit_logs_user_time ON audit_logs(user_id, timestamp DESC);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);

-- Retention policy: Keep 2 years, then archive to cold storage
CREATE OR REPLACE FUNCTION archive_old_audit_logs()
RETURNS void AS $$
BEGIN
  -- Export to S3 and drop partitions older than 2 years
  PERFORM export_and_drop_partition(
    'audit_logs',
    NOW() - INTERVAL '2 years'
  );
END;
$$ LANGUAGE plpgsql;
```

**3. Performance Optimizations**

**Advanced Caching Strategy:**
```javascript
// Multi-level cache
class CacheManager {
  constructor() {
    this.l1 = new LRUCache({ max: 1000, ttl: 60000 }); // 1 min
    this.l2 = new RedisCache({ ttl: 900 }); // 15 min
  }

  async get(key) {
    // Try L1 cache (in-memory)
    let value = this.l1.get(key);
    if (value) {
      return value;
    }

    // Try L2 cache (Redis)
    value = await this.l2.get(key);
    if (value) {
      this.l1.set(key, value);
      return value;
    }

    // Cache miss
    return null;
  }

  async set(key, value, options = {}) {
    this.l1.set(key, value);
    await this.l2.set(key, value, options);
  }

  async invalidate(pattern) {
    this.l1.clear();
    await this.l2.deletePattern(pattern);
  }
}
```

**Query Optimization:**
```sql
-- Materialized views for expensive aggregations
CREATE MATERIALIZED VIEW model_stats AS
SELECT
  organization_id,
  framework,
  COUNT(*) as total_models,
  AVG((metadata->>'size_mb')::numeric) as avg_size_mb,
  MAX(created_at) as latest_model_date
FROM models
WHERE deleted_at IS NULL
GROUP BY organization_id, framework;

-- Refresh strategy
CREATE INDEX idx_model_stats_org ON model_stats(organization_id);
REFRESH MATERIALIZED VIEW CONCURRENTLY model_stats;

-- Scheduled refresh (every hour)
SELECT cron.schedule('refresh-model-stats', '0 * * * *',
  $$REFRESH MATERIALIZED VIEW CONCURRENTLY model_stats$$
);
```

**Connection Pooling:**
```javascript
// PgBouncer configuration
[databases]
llmregistry = host=db-primary port=5432 dbname=llmregistry

[pgbouncer]
pool_mode = transaction
max_client_conn = 10000
default_pool_size = 100
reserve_pool_size = 25
reserve_pool_timeout = 5

// Application connection pool
const pool = new Pool({
  host: 'pgbouncer.db.internal',
  port: 6432,
  database: 'llmregistry',
  max: 20, // Max connections per app instance
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 2000
});
```

**Database Indexing (Advanced):**
```sql
-- Covering indexes for common queries
CREATE INDEX idx_models_covering ON models(
  organization_id,
  framework,
  status,
  created_at DESC
) INCLUDE (name, version, tags);

-- Partial indexes for hot data
CREATE INDEX idx_models_active ON models(organization_id, created_at DESC)
WHERE status = 'active' AND deleted_at IS NULL;

-- Expression indexes
CREATE INDEX idx_models_name_lower ON models(LOWER(name));

-- Index usage monitoring
SELECT
  schemaname,
  tablename,
  indexname,
  idx_scan,
  idx_tup_read,
  idx_tup_fetch
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
ORDER BY idx_scan ASC;
```

**4. Production Monitoring and Metrics**

**Observability Stack:**
```
┌──────────────────────────────────────┐
│        Application Metrics           │
│        (Prometheus)                  │
└───────────┬──────────────────────────┘
            │
┌───────────▼──────────────────────────┐
│        Visualization                 │
│        (Grafana)                     │
└──────────────────────────────────────┘

┌──────────────────────────────────────┐
│        Distributed Tracing           │
│        (Jaeger)                      │
└──────────────────────────────────────┘

┌──────────────────────────────────────┐
│        Log Aggregation               │
│        (ELK Stack)                   │
└──────────────────────────────────────┘

┌──────────────────────────────────────┐
│        Alerting                      │
│        (PagerDuty)                   │
└──────────────────────────────────────┘
```

**Metrics Collection:**
```javascript
// Prometheus metrics
const prometheus = require('prom-client');

// API request metrics
const httpRequestDuration = new prometheus.Histogram({
  name: 'http_request_duration_seconds',
  help: 'Duration of HTTP requests in seconds',
  labelNames: ['method', 'route', 'status_code'],
  buckets: [0.01, 0.05, 0.1, 0.5, 1, 5]
});

// Database query metrics
const dbQueryDuration = new prometheus.Histogram({
  name: 'db_query_duration_seconds',
  help: 'Duration of database queries',
  labelNames: ['query_type', 'table'],
  buckets: [0.001, 0.01, 0.05, 0.1, 0.5, 1]
});

// Business metrics
const modelsRegistered = new prometheus.Counter({
  name: 'models_registered_total',
  help: 'Total number of models registered',
  labelNames: ['organization_id', 'framework']
});

// Cache hit rate
const cacheHits = new prometheus.Counter({
  name: 'cache_hits_total',
  help: 'Total cache hits',
  labelNames: ['cache_level']
});

const cacheMisses = new prometheus.Counter({
  name: 'cache_misses_total',
  help: 'Total cache misses',
  labelNames: ['cache_level']
});
```

**Grafana Dashboards:**
```yaml
# Key metrics dashboard
dashboard:
  title: "LLM Registry - Overview"
  panels:
    - title: "Requests per Second"
      targets:
        - expr: rate(http_requests_total[5m])

    - title: "API Latency (p95)"
      targets:
        - expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

    - title: "Error Rate"
      targets:
        - expr: rate(http_requests_total{status_code=~"5.."}[5m])

    - title: "Database Connection Pool"
      targets:
        - expr: pg_pool_connections_active / pg_pool_connections_total

    - title: "Cache Hit Rate"
      targets:
        - expr: cache_hits_total / (cache_hits_total + cache_misses_total)

    - title: "Models Registered (24h)"
      targets:
        - expr: increase(models_registered_total[24h])
```

**Alerting Rules:**
```yaml
# Prometheus alerting rules
groups:
  - name: llm-registry
    interval: 30s
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status_code=~"5.."}[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} (>5%)"

      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 1
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High API latency"
          description: "p95 latency is {{ $value }}s (>1s)"

      - alert: DatabaseReplicationLag
        expr: pg_replication_lag_seconds > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Database replication lag high"
          description: "Replication lag is {{ $value }}s"

      - alert: LowCacheHitRate
        expr: cache_hits_total / (cache_hits_total + cache_misses_total) < 0.7
        for: 15m
        labels:
          severity: info
        annotations:
          summary: "Low cache hit rate"
          description: "Cache hit rate is {{ $value }} (<70%)"
```

**Distributed Tracing:**
```javascript
// OpenTelemetry instrumentation
const { NodeTracerProvider } = require('@opentelemetry/sdk-trace-node');
const { JaegerExporter } = require('@opentelemetry/exporter-jaeger');

const provider = new NodeTracerProvider();
provider.addSpanProcessor(
  new BatchSpanProcessor(
    new JaegerExporter({
      endpoint: 'http://jaeger:14268/api/traces'
    })
  )
);

// Trace requests
app.use((req, res, next) => {
  const span = tracer.startSpan('http_request', {
    attributes: {
      'http.method': req.method,
      'http.url': req.url,
      'http.user_agent': req.headers['user-agent']
    }
  });

  res.on('finish', () => {
    span.setAttribute('http.status_code', res.statusCode);
    span.end();
  });

  next();
});
```

**5. Comprehensive Documentation**

**Documentation Structure:**
```
docs/
├── README.md
├── getting-started/
│   ├── quickstart.md
│   ├── installation.md
│   └── authentication.md
├── api-reference/
│   ├── rest-api.md
│   ├── graphql-api.md
│   └── webhooks.md
├── guides/
│   ├── model-registration.md
│   ├── lineage-tracking.md
│   ├── multi-tenancy.md
│   └── rbac-setup.md
├── integrations/
│   ├── mlflow.md
│   ├── wandb.md
│   ├── huggingface.md
│   └── tensorboard.md
├── deployment/
│   ├── kubernetes.md
│   ├── docker-compose.md
│   ├── aws.md
│   └── gcp.md
├── operations/
│   ├── monitoring.md
│   ├── backup-restore.md
│   ├── scaling.md
│   └── troubleshooting.md
└── contributing/
    ├── development-setup.md
    ├── code-style.md
    └── release-process.md
```

**Interactive API Documentation:**
```yaml
# OpenAPI 3.0 specification
openapi: 3.0.0
info:
  title: LLM Registry API
  version: 1.0.0
  description: |
    The LLM Registry API provides a centralized system for managing
    machine learning model metadata, lineage, and lifecycle.

servers:
  - url: https://api.llm-registry.com/v1
    description: Production
  - url: https://staging-api.llm-registry.com/v1
    description: Staging

paths:
  /models:
    get:
      summary: List models
      description: Retrieve a paginated list of models
      parameters:
        - name: framework
          in: query
          schema:
            type: string
            enum: [pytorch, tensorflow, onnx]
        - name: limit
          in: query
          schema:
            type: integer
            default: 50
            maximum: 100
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ModelListResponse'
              examples:
                pytorch-models:
                  summary: PyTorch models
                  value:
                    data: [...]
                    pagination: {...}

components:
  schemas:
    Model:
      type: object
      required: [id, name, framework, version]
      properties:
        id:
          type: string
          format: uuid
        name:
          type: string
        framework:
          type: string
          enum: [pytorch, tensorflow, onnx]
```

**SDK Documentation:**
```python
# Python SDK with type hints
from typing import Optional, List, Dict
from llm_registry import LLMRegistry, Model, ModelFilter

# Initialize client
registry = LLMRegistry(
    api_url="https://api.llm-registry.com",
    api_key="llmreg_sk_live_..."
)

# Register a model
model = registry.register_model(
    name="gpt-classifier-v2",
    framework="pytorch",
    version="2.0.0",
    tags=["nlp", "classification"],
    metadata={
        "accuracy": 0.96,
        "f1_score": 0.94
    }
)

# Query models
models = registry.list_models(
    filter=ModelFilter(
        framework="pytorch",
        tags=["nlp"],
        created_after="2025-01-01"
    ),
    limit=50
)

# Get lineage
lineage = registry.get_model_lineage(
    model_id="model_123",
    depth=2
)
```

### Dependencies and Prerequisites

**Infrastructure (Production-grade):**
- [ ] Kubernetes cluster (multi-region)
- [ ] Distributed database cluster (Citus or CockroachDB)
- [ ] Redis cluster (3+ nodes)
- [ ] Object storage (S3 with cross-region replication)
- [ ] CDN (CloudFront or Cloudflare)
- [ ] Monitoring stack (Prometheus, Grafana, Jaeger, ELK)
- [ ] Secret management (Vault or AWS Secrets Manager)

**Security:**
- [ ] WAF (Web Application Firewall)
- [ ] DDoS protection
- [ ] SSL/TLS certificates with auto-renewal
- [ ] Security scanning (Snyk, SonarQube)
- [ ] Penetration testing completed

**Compliance:**
- [ ] SOC 2 Type II certification (if applicable)
- [ ] GDPR compliance review
- [ ] Data retention policies implemented
- [ ] Privacy policy and terms of service

**Team (Full production team):**
- [ ] 5-6 backend engineers
- [ ] 2-3 frontend engineers
- [ ] 2 DevOps/SRE engineers
- [ ] 1 security engineer
- [ ] 2 technical writers
- [ ] 1 product manager
- [ ] 1 engineering manager

### Validation Metrics and Success Criteria

**Functional Metrics:**
- [ ] 100% feature completeness vs roadmap
- [ ] All ecosystem integrations stable
- [ ] Advanced RBAC tested with 20+ permission scenarios
- [ ] Audit logging capturing 100% of write operations

**Performance Metrics (SLAs):**
- [ ] API response time: <50ms (p95) for GET, <200ms for POST
- [ ] Complex lineage queries: <300ms (p95)
- [ ] Support 10,000+ concurrent users
- [ ] Handle 1M+ models in registry
- [ ] Read throughput: 50,000 queries/second
- [ ] Write throughput: 5,000 writes/second

**Reliability Metrics:**
- [ ] 99.9% uptime (max 43 minutes downtime/month)
- [ ] Automatic failover: <30 seconds
- [ ] Zero data loss guarantee (RPO = 0)
- [ ] Recovery time: <5 minutes (RTO)
- [ ] Database replication lag: <10ms (p95)

**Security Metrics:**
- [ ] Zero critical security vulnerabilities
- [ ] All dependencies up to date
- [ ] Penetration test: No high-severity findings
- [ ] 100% API requests authenticated
- [ ] Rate limiting: 99.9% accuracy

**User Metrics:**
- [ ] 500+ production organizations
- [ ] 95% user satisfaction
- [ ] <1% API error rate
- [ ] >90% cache hit rate
- [ ] Documentation: >90% coverage

### Timeline

**Weeks 1-6: Distributed Infrastructure**
- Week 1-2: Kubernetes cluster setup
- Week 3-4: Distributed database (Citus/CockroachDB)
- Week 5-6: Multi-region replication

**Weeks 7-12: Advanced Features**
- Week 7-8: Advanced RBAC implementation
- Week 9-10: Comprehensive audit logging
- Week 11-12: Performance optimizations (caching, indexing)

**Weeks 13-16: Monitoring & Observability**
- Week 13: Prometheus + Grafana setup
- Week 14: Distributed tracing (Jaeger)
- Week 15: Log aggregation (ELK)
- Week 16: Alerting and on-call setup

**Weeks 17-20: Documentation & Hardening**
- Week 17-18: Complete documentation
- Week 18-19: Security hardening, penetration testing
- Week 19: Disaster recovery testing
- Week 20: Load testing (stress tests)

**Weeks 21-24: Release Preparation**
- Week 21: Beta migration to v1
- Week 22: Production deployment (phased rollout)
- Week 23: Monitoring and optimization
- Week 24: v1.0 GA release

**Milestones:**
- [ ] Week 6: Infrastructure ready
- [ ] Week 12: All features complete
- [ ] Week 16: Observability complete
- [ ] Week 20: Documentation complete
- [ ] Week 24: v1.0 GA

### Scaling Considerations

**v1 Scale Targets:**
- **Users**: 500-5,000 organizations
- **Models**: 100,000-1,000,000+ models
- **Traffic**: 10,000-50,000 requests/second peak
- **Storage**: 100TB total
- **Concurrent users**: 10,000+

**Horizontal Scaling:**
- **API servers**: Auto-scale from 5 to 100 pods
- **Database**: Shard by organization_id
- **Cache**: Redis cluster with 10+ nodes
- **Background jobs**: Dedicated worker pools

**Vertical Scaling:**
- **Database**: Scale to 32+ vCPUs, 128GB+ RAM per node
- **API servers**: Scale to 4 vCPUs, 8GB RAM per pod
- **Cache nodes**: 16GB+ RAM per node

**Geographic Distribution:**
- **3+ regions**: US-East, US-West, EU-West
- **Edge caching**: CloudFront with 50+ POPs
- **Data residency**: Region-specific data storage

**Cost Optimization:**
- **Reserved instances**: 30-50% cost savings
- **Spot instances**: For non-critical workloads
- **Storage tiering**: S3 Intelligent-Tiering
- **Compression**: Reduce storage and bandwidth costs

---

## Cross-Phase Considerations

### Migration Strategy

**MVP → Beta:**
- [ ] Data migration scripts for schema changes
- [ ] Backward compatibility for API clients
- [ ] Gradual rollout with feature flags
- [ ] Rollback plan if issues detected

**Beta → v1:**
- [ ] Zero-downtime migration strategy
- [ ] Blue-green deployment
- [ ] Database migration with online schema changes
- [ ] Client SDK update notifications

### Backward Compatibility

**API Versioning:**
- Support previous API version for minimum 12 months
- Clear deprecation notices 6 months in advance
- Migration guides and automated tools

**Database Migrations:**
- Use online schema migration tools (gh-ost, pt-online-schema-change)
- Test migrations on production-sized datasets
- Maintain backward-compatible schemas during transition

### Risk Mitigation

**Technical Risks:**
- **Database bottlenecks**: Addressed via sharding, replication
- **API performance**: Mitigated with caching, CDN
- **Data loss**: Prevented with backups, replication
- **Security breaches**: Mitigated with audits, monitoring

**Operational Risks:**
- **Team scaling**: Hire early, cross-train
- **Vendor lock-in**: Use open standards, multi-cloud
- **Compliance**: Engage legal/compliance early

**Business Risks:**
- **User adoption**: Focus on integrations, developer experience
- **Competition**: Differentiate with superior lineage tracking
- **Pricing**: Validate with beta users

---

## Success Metrics Summary

| Metric | MVP | Beta | v1 |
|--------|-----|------|-----|
| Uptime SLA | 95% | 99% | 99.9% |
| API Latency (p95) | <200ms | <100ms | <50ms |
| Concurrent Users | 100 | 1,000 | 10,000+ |
| Models Supported | 10,000 | 100,000 | 1,000,000+ |
| Throughput (req/s) | 100 | 1,000 | 10,000+ |
| Organizations | 10-50 | 50-500 | 500-5,000+ |
| Integrations | 2 | 4 | 6+ |
| Documentation | Basic | Comprehensive | Complete + Videos |
| Security | Basic auth | Multi-tenant + RBAC | Advanced RBAC + Audit |
| Monitoring | Basic logs | Metrics + Alerts | Full observability |

---

## Conclusion

This phased roadmap provides a clear path from MVP to production-ready v1:

1. **MVP (12-16 weeks)**: Focus on core functionality, single-node deployment, and 1-2 integrations. Validate the concept with early adopters.

2. **Beta (16-20 weeks)**: Add advanced features (lineage, multi-tenancy, GraphQL), expand to 4 integrations, and implement 2-node replication for reliability.

3. **v1 (20-24 weeks)**: Deliver production-grade distributed system with advanced RBAC, comprehensive monitoring, and full documentation. Ready for enterprise adoption.

Each phase builds on the previous, with clear validation metrics and success criteria. The roadmap balances feature development with operational maturity, ensuring the system is ready for production workloads by v1.
