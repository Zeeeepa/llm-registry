# LLM-Registry Specification

## 1. System Purpose Statement

The **LLM-Registry** serves as the central metadata repository and catalog system for the LLM ecosystem, providing authoritative tracking, versioning, and discovery of LLM models, pipelines, configurations, and associated artifacts. It acts as a single source of truth for model metadata, lineage, dependencies, and compliance status across all ecosystem components.

### Core Objectives

- **Centralized Metadata Management**: Maintain comprehensive, versioned metadata for all LLM models, pipelines, and configurations
- **Lineage Tracking**: Record and query complete dependency graphs and transformation lineages
- **Policy Association**: Link models and pipelines to governance policies and track compliance status
- **Service Discovery**: Enable components to discover and retrieve model configurations, endpoints, and capabilities
- **Audit Trail**: Provide immutable audit logs of all model registrations, updates, and policy validations

---

## 2. Functional Requirements

### 2.1 Model Metadata Management

#### FR-1.1: Model Registration
- **Description**: Support registration of new model entries with comprehensive metadata
- **Inputs**:
  - Model name (unique identifier)
  - Version (semantic versioning)
  - Provider (e.g., OpenAI, Anthropic, Cohere, HuggingFace, custom)
  - Model type (completion, embedding, vision, function-calling)
  - Checksum/hash (SHA-256) for artifact integrity
  - Endpoint URLs and authentication requirements
  - Capability metadata (context length, supported features)
- **Outputs**:
  - Unique registry ID
  - Timestamp of registration
  - Initial version record
- **Acceptance Criteria**:
  - Registration completes in <200ms (p95)
  - Duplicate detection prevents re-registration of identical models
  - Validation ensures all required fields are present and well-formed

#### FR-1.2: Model Versioning
- **Description**: Track complete version history with semantic versioning
- **Requirements**:
  - Support major.minor.patch versioning scheme
  - Maintain changelog with version deltas
  - Enable version aliasing (e.g., "latest", "stable", "production")
  - Support version deprecation with sunset dates
  - Provide version comparison and compatibility checks
- **Constraints**:
  - Versions are immutable once published
  - Version history must be retained indefinitely
  - Breaking changes require major version increment

#### FR-1.3: Model Discovery and Query
- **Description**: Enable efficient search and discovery of registered models
- **Query Capabilities**:
  - Search by name, provider, type, tags
  - Filter by version constraints (e.g., ">=2.0.0")
  - Query by capabilities (e.g., context_length > 8000)
  - Semantic search on descriptions and tags
  - Full-text search across metadata fields
- **Performance Requirements**:
  - Query latency <50ms (p95) for indexed fields
  - Support pagination with offset/limit
  - Return results sorted by relevance or custom criteria

#### FR-1.4: Model Metadata Update
- **Description**: Support updates to mutable metadata fields
- **Mutable Fields**:
  - Description and documentation
  - Tags and labels
  - Endpoint URLs
  - Status (active, deprecated, retired)
  - Custom metadata extensions
- **Immutable Fields**:
  - Model name and version
  - Provider
  - Checksum
  - Registration timestamp
- **Constraints**:
  - Updates create audit log entries
  - Critical updates (endpoint changes) trigger notifications
  - Updates are atomic and transactional

### 2.2 Pipeline Configuration Storage

#### FR-2.1: Pipeline Registration
- **Description**: Store complete pipeline definitions including stages, transforms, and configurations
- **Schema Requirements**:
  - Pipeline ID and version
  - Directed Acyclic Graph (DAG) of pipeline stages
  - Model references for each stage
  - Transform and preprocessing configurations
  - Input/output schema definitions
  - Resource requirements (compute, memory, GPU)
  - Timeout and retry policies
- **Validation**:
  - DAG validation (no cycles)
  - Model reference validation (all models exist in registry)
  - Schema compatibility checks
  - Resource constraint validation

#### FR-2.2: Pipeline Versioning
- **Description**: Version control for pipeline configurations
- **Requirements**:
  - Track changes to pipeline structure
  - Support pipeline cloning and branching
  - Enable rollback to previous versions
  - Maintain compatibility matrix with model versions

#### FR-2.3: Pipeline Templates
- **Description**: Support reusable pipeline templates
- **Features**:
  - Template parameterization
  - Template inheritance and composition
  - Validation of template instantiations
  - Template marketplace integration

### 2.3 Test Suite Definitions

#### FR-3.1: Test Suite Registration
- **Description**: Store test suite definitions for model and pipeline validation
- **Components**:
  - Test suite ID and version
  - Test case definitions (inputs, expected outputs, assertions)
  - Performance benchmarks (latency, throughput)
  - Quality metrics (accuracy, precision, recall)
  - Compliance checks (safety, bias, toxicity)
- **Test Types**:
  - Unit tests (individual model behavior)
  - Integration tests (pipeline end-to-end)
  - Performance tests (load and stress)
  - Compliance tests (policy validation)

#### FR-3.2: Test Result Storage
- **Description**: Record and query test execution results
- **Data Captured**:
  - Test suite version
  - Model/pipeline version tested
  - Execution timestamp and duration
  - Pass/fail status per test case
  - Performance metrics
  - Failure diagnostics
- **Query Capabilities**:
  - Query results by model/pipeline version
  - Trend analysis over time
  - Failure pattern detection

### 2.4 Policy Association and Compliance Tracking

#### FR-4.1: Policy Linking
- **Description**: Associate governance policies with models and pipelines
- **Capabilities**:
  - Link multiple policies to single model/pipeline
  - Support policy inheritance (global, team, project)
  - Enable policy version tracking
  - Record policy exceptions and approvals

#### FR-4.2: Compliance Status Tracking
- **Description**: Maintain real-time compliance status for all registered entities
- **Status Types**:
  - `compliant`: Passes all associated policies
  - `non-compliant`: Fails one or more policies
  - `pending`: Awaiting policy validation
  - `exempt`: Has approved exceptions
  - `unknown`: Not yet evaluated
- **Requirements**:
  - Real-time status updates from LLM-Policy-Engine
  - Compliance history with timestamps
  - Alert on compliance state changes
  - Support compliance reporting and dashboards

#### FR-4.3: Compliance Reports
- **Description**: Generate compliance reports for audit and governance
- **Report Types**:
  - Compliance summary by policy
  - Non-compliant models list
  - Trend analysis
  - Exception audit logs
- **Formats**: JSON, CSV, PDF

### 2.5 Dependency and Lineage Management

#### FR-5.1: Dependency Graph Construction
- **Description**: Build and maintain dependency graphs for models and pipelines
- **Graph Structure**:
  - Nodes: Models, pipelines, datasets, configurations
  - Edges: Dependencies, transformations, derivations
  - Metadata: Dependency type, version constraints
- **Capabilities**:
  - Detect circular dependencies
  - Compute transitive dependencies
  - Support version pinning and ranges
  - Impact analysis for updates

#### FR-5.2: Lineage Tracking
- **Description**: Record complete lineage from raw data through model training to deployment
- **Lineage Types**:
  - Data lineage (datasets → preprocessed data → training data)
  - Model lineage (base model → fine-tuned model → deployed model)
  - Pipeline lineage (stages and transformations)
- **Integration**:
  - Sync lineage with LLM-Memory-Graph for graph storage
  - Enable temporal lineage queries
  - Support lineage visualization

#### FR-5.3: Impact Analysis
- **Description**: Analyze impact of changes to models, pipelines, or dependencies
- **Capabilities**:
  - Identify downstream consumers of a model
  - Calculate blast radius of breaking changes
  - Generate migration plans for version upgrades
  - Detect orphaned dependencies

### 2.6 Extensibility and Custom Metadata

#### FR-6.1: Custom Metadata Schema
- **Description**: Support user-defined metadata schemas
- **Requirements**:
  - JSON Schema validation for custom fields
  - Namespace isolation for custom metadata
  - Indexing support for custom fields
  - Query custom fields with type-safe operators

#### FR-6.2: Plugin Architecture
- **Description**: Enable plugins for custom functionality
- **Plugin Types**:
  - Metadata validators
  - Custom query operators
  - Lifecycle hooks (pre/post registration)
  - Custom reporting
- **Requirements**:
  - Plugin isolation (sandboxing)
  - Versioned plugin API
  - Plugin discovery and management

---

## 3. Non-Functional Requirements

### 3.1 Performance

#### NFR-1.1: Query Latency
- **Requirement**: p95 query latency <50ms for indexed queries
- **Measurement**: Time from request receipt to response sent
- **SLO**: 99.5% of indexed queries complete within 50ms
- **Optimization Strategies**:
  - Multi-level caching (in-memory, distributed)
  - Index optimization (B-tree, inverted index)
  - Query plan optimization
  - Read replicas for load distribution

#### NFR-1.2: Write Throughput
- **Requirement**: Support 1000+ writes/second per node
- **Constraints**:
  - Write latency p95 <100ms
  - Strong consistency for critical paths
  - Eventual consistency for non-critical metadata
- **Scaling Strategy**:
  - Write partitioning by namespace
  - Async replication
  - Write buffering and batching

#### NFR-1.3: Concurrent Operations
- **Requirement**: Support 10,000+ concurrent read operations
- **Strategies**:
  - Connection pooling
  - Read-through caching
  - Load balancing
  - Circuit breakers for fault isolation

### 3.2 Scalability

#### NFR-2.1: Data Volume
- **Capacity**: Support 1M+ model entries, 100K+ pipelines
- **Growth**: Scale to 10M+ entries without re-architecture
- **Storage**: Auto-scaling storage based on usage patterns

#### NFR-2.2: Horizontal Scalability
- **Architecture**: Stateless service layer, distributed data layer
- **Sharding**:
  - Shard by namespace/tenant
  - Consistent hashing for shard assignment
  - Shard rebalancing on node addition/removal
- **Replication**:
  - Multi-region replication for disaster recovery
  - Read replicas per region
  - Quorum-based writes

#### NFR-2.3: Query Scalability
- **Requirement**: Query performance remains <100ms as data grows to 10M+ entries
- **Strategies**:
  - Partitioned indexes
  - Query result caching
  - Materialized views for common queries
  - Search index optimization (Elasticsearch/OpenSearch)

### 3.3 Reliability

#### NFR-3.1: Availability
- **SLA**: 99.95% uptime (26 minutes downtime/year)
- **Deployment Strategy**:
  - Multi-zone deployment
  - Zero-downtime rolling updates
  - Canary deployments
  - Automated rollback on failure

#### NFR-3.2: Data Consistency
- **Consistency Model**:
  - Strong consistency for writes (ACID)
  - Tunable consistency for reads (eventual or strong)
  - Linearizability for critical operations
- **Conflict Resolution**:
  - Last-write-wins for commutative updates
  - CRDT for metadata counters
  - Manual resolution for conflicts

#### NFR-3.3: Data Integrity
- **Requirements**:
  - Checksum validation on all stored artifacts
  - Replication verification
  - Corruption detection and repair
  - Backup verification (monthly)
- **Audit**:
  - Immutable audit logs
  - Tamper detection
  - Retention policy (7 years minimum)

#### NFR-3.4: Fault Tolerance
- **Recovery Objectives**:
  - RTO (Recovery Time Objective): <5 minutes
  - RPO (Recovery Point Objective): <1 minute
- **Strategies**:
  - Automated failover
  - Data replication (3+ copies)
  - Point-in-time recovery
  - Disaster recovery drills (quarterly)

### 3.4 Security

#### NFR-4.1: Authentication
- **Requirements**:
  - Support OAuth 2.0 / OIDC for user authentication
  - API key authentication for service accounts
  - Mutual TLS (mTLS) for service-to-service
  - Token expiration and rotation
- **Identity Providers**:
  - Integration with enterprise SSO (SAML, LDAP)
  - Support multi-factor authentication (MFA)

#### NFR-4.2: Authorization
- **Model**: Role-Based Access Control (RBAC) + Attribute-Based Access Control (ABAC)
- **Permissions**:
  - `registry:read` - Query registry
  - `registry:write` - Register/update models
  - `registry:delete` - Delete models (soft delete)
  - `registry:admin` - Administrative operations
- **Scoping**:
  - Global, namespace, and resource-level permissions
  - Dynamic permission evaluation
  - Permission inheritance

#### NFR-4.3: Multi-Tenancy
- **Isolation**:
  - Logical isolation by namespace/tenant ID
  - Row-level security in data layer
  - Separate encryption keys per tenant
- **Resource Quotas**:
  - Configurable limits per tenant (models, pipelines, storage)
  - Rate limiting per tenant
  - Cost tracking and chargeback

#### NFR-4.4: Data Encryption
- **At Rest**:
  - AES-256 encryption for all stored data
  - Key rotation (quarterly)
  - Hardware Security Module (HSM) for key management
- **In Transit**:
  - TLS 1.3 for all network communication
  - Certificate pinning for critical services
  - End-to-end encryption for sensitive metadata

#### NFR-4.5: Audit Logging
- **Requirements**:
  - Log all read/write operations
  - Capture user identity, timestamp, operation, resource
  - Tamper-proof audit trail
  - Real-time security event monitoring
- **Retention**: 7 years minimum for compliance

### 3.5 Observability

#### NFR-5.1: Metrics
- **System Metrics**:
  - Request rate, latency (p50, p95, p99)
  - Error rate by status code
  - Throughput (ops/second)
  - Resource utilization (CPU, memory, disk, network)
- **Business Metrics**:
  - Models registered per day
  - Query patterns and frequency
  - Compliance status distribution
- **Export**: Prometheus format, 15-second scrape interval

#### NFR-5.2: Logging
- **Structured Logging**: JSON format with standard fields
- **Log Levels**: DEBUG, INFO, WARN, ERROR, CRITICAL
- **Correlation**: Request ID propagation across services
- **Retention**: 30 days hot, 1 year cold storage

#### NFR-5.3: Tracing
- **Distributed Tracing**: OpenTelemetry integration
- **Span Coverage**: All external calls, database queries, cache operations
- **Sampling**: 100% for errors, 1% for successful requests
- **Storage**: Jaeger or Tempo backend

#### NFR-5.4: Alerting
- **Alert Types**:
  - Service health (uptime, error rate)
  - Performance degradation (latency SLO breach)
  - Security events (failed auth, permission violations)
  - Data integrity (checksum failures, replication lag)
- **Channels**: PagerDuty, Slack, email
- **SLA**: P1 alerts <5 min response time

---

## 4. Integration Requirements

### 4.1 LLM-Memory-Graph Integration

#### INT-1.1: Lineage Graph Synchronization
- **Purpose**: Sync dependency and lineage graphs to LLM-Memory-Graph for advanced graph queries
- **Data Flow**:
  - Registry → Memory-Graph: Push lineage updates on model/pipeline registration
  - Memory-Graph → Registry: Query expanded lineage graphs
- **Protocol**: gRPC for high-performance streaming
- **Events**:
  - `model.registered`: Publish model node to graph
  - `dependency.created`: Publish edge to graph
  - `lineage.updated`: Sync lineage changes
- **Consistency**: Eventually consistent (sync lag <1 second)

#### INT-1.2: Contextual Relationship Queries
- **Purpose**: Enable semantic queries over model relationships
- **Capabilities**:
  - "Find all models derived from base model X"
  - "What pipelines depend on this model?"
  - "Show transformation path from dataset A to model B"
- **API**: GraphQL endpoint proxying to Memory-Graph
- **Caching**: Cache frequent graph queries (TTL 5 minutes)

#### INT-1.3: Versioned Graph State
- **Purpose**: Query graph state at specific points in time
- **Requirements**:
  - Support temporal queries ("show dependencies as of timestamp T")
  - Maintain historical graph snapshots
  - Efficient delta storage for version history

### 4.2 LLM-Policy-Engine Integration

#### INT-2.1: Policy Validation Hooks
- **Purpose**: Trigger policy validation on model registration/update
- **Workflow**:
  1. Registry receives model registration request
  2. Registry publishes `model.pre-register` event to Policy Engine
  3. Policy Engine validates against applicable policies
  4. Policy Engine responds with validation result
  5. Registry proceeds or rejects based on result
- **Timeout**: 5 seconds max for policy validation
- **Fallback**: Configurable behavior on timeout (reject or allow with warning)

#### INT-2.2: Compliance Status Updates
- **Purpose**: Receive real-time compliance status updates from Policy Engine
- **Event Stream**: Subscribe to `compliance.status-changed` events
- **Update Handling**:
  - Update compliance status in registry metadata
  - Trigger notifications for compliance violations
  - Update compliance dashboard metrics
- **Protocol**: WebSocket or Server-Sent Events (SSE)

#### INT-2.3: Policy Metadata Queries
- **Purpose**: Query policy definitions and requirements from Policy Engine
- **Use Cases**:
  - Display policy requirements during model registration
  - Validate metadata against policy schemas
  - Generate compliance reports
- **API**: REST API with caching (TTL 15 minutes)

### 4.3 LLM-Forge Integration

#### INT-3.1: SDK Metadata Synchronization
- **Purpose**: Sync model metadata to LLM-Forge SDK for code generation
- **Data Sync**:
  - Model schemas (input/output types)
  - API endpoint configurations
  - Authentication requirements
  - Rate limits and quotas
- **Sync Frequency**: Real-time on model update, periodic full sync (hourly)
- **Protocol**: REST API with webhook callbacks

#### INT-3.2: Client Library Generation
- **Purpose**: Provide metadata for automated client library generation
- **Metadata Provided**:
  - OpenAPI/Swagger specifications
  - gRPC proto definitions
  - GraphQL schemas
  - Authentication flows
- **Versioning**: Client libraries track model versions
- **Distribution**: Publish to package registries (npm, PyPI, Maven)

#### INT-3.3: Configuration Templates
- **Purpose**: Share configuration templates with LLM-Forge
- **Template Types**:
  - Model invocation templates
  - Pipeline DAG templates
  - Deployment configurations
- **Format**: JSON/YAML with Jinja2/Handlebars templating
- **Validation**: Template validation before publication

### 4.4 LLM-Governance-Dashboard Integration

#### INT-4.1: Metrics Export
- **Purpose**: Export registry metrics to Governance Dashboard
- **Metrics Exported**:
  - Model registration trends
  - Compliance status distribution
  - Query patterns and usage analytics
  - Dependency complexity metrics
- **Protocol**: Push metrics via Prometheus pushgateway or REST API
- **Frequency**: Real-time stream + periodic aggregates (hourly, daily)

#### INT-4.2: Dashboard Data API
- **Purpose**: Provide read API for dashboard queries
- **Endpoints**:
  - `GET /api/v1/dashboard/models` - Model catalog with filters
  - `GET /api/v1/dashboard/compliance` - Compliance summary
  - `GET /api/v1/dashboard/lineage/{id}` - Lineage visualization data
  - `GET /api/v1/dashboard/analytics` - Analytics queries
- **Performance**: <200ms p95 for dashboard queries
- **Caching**: Aggressive caching with stale-while-revalidate

#### INT-4.3: Admin Operations API
- **Purpose**: Enable administrative operations from dashboard
- **Operations**:
  - Model approval workflows
  - Policy exception approvals
  - Bulk metadata updates
  - Registry configuration
- **Authorization**: Admin-level permissions required
- **Audit**: All operations logged to audit trail

#### INT-4.4: Real-Time Updates
- **Purpose**: Push real-time updates to dashboard
- **Events**:
  - Model registration/updates
  - Compliance status changes
  - Policy violations
  - System health alerts
- **Protocol**: WebSocket with reconnection handling
- **Backpressure**: Client-side buffering and throttling

---

## 5. Data Schema Specifications

### 5.1 Core Model Schema

```typescript
interface ModelMetadata {
  // Identity
  id: string;                    // UUID v4
  name: string;                  // Unique model name
  version: SemanticVersion;      // major.minor.patch
  provider: ModelProvider;       // OpenAI, Anthropic, etc.

  // Classification
  type: ModelType;               // completion, embedding, vision, function
  category: string[];            // [general, code, reasoning, etc.]
  tags: string[];                // User-defined tags

  // Technical Metadata
  checksum: string;              // SHA-256 hash
  checksumAlgorithm: 'sha256';
  artifactUri?: string;          // Optional model artifact location

  // Capabilities
  capabilities: ModelCapabilities;

  // Endpoints
  endpoints: ModelEndpoint[];

  // Versioning
  versionMetadata: VersionMetadata;

  // Lifecycle
  status: ModelStatus;           // active, deprecated, retired
  deprecationDate?: Date;
  retirementDate?: Date;

  // Ownership
  owner: string;                 // Owner user/team ID
  namespace: string;             // Tenant/namespace

  // Timestamps
  createdAt: Date;
  updatedAt: Date;
  registeredBy: string;          // User ID who registered

  // Custom Metadata
  customMetadata: Record<string, unknown>;

  // Relations
  dependencies: ModelDependency[];
  derivedFrom?: string;          // Parent model ID

  // Compliance
  policyAssociations: PolicyAssociation[];
  complianceStatus: ComplianceStatus;
}

type ModelProvider =
  | 'openai'
  | 'anthropic'
  | 'google'
  | 'cohere'
  | 'huggingface'
  | 'custom';

type ModelType =
  | 'completion'
  | 'embedding'
  | 'vision'
  | 'function-calling'
  | 'multi-modal';

interface ModelCapabilities {
  contextLength: number;         // Max context window
  maxOutputTokens: number;
  supportsFunctions: boolean;
  supportsVision: boolean;
  supportsStreaming: boolean;
  supportsBatching: boolean;
  languages: string[];           // Supported languages
  modalities: string[];          // text, image, audio, video
  features: string[];            // Custom features
}

interface ModelEndpoint {
  type: 'rest' | 'grpc' | 'graphql';
  url: string;
  authentication: AuthenticationConfig;
  rateLimit?: RateLimit;
  timeout?: number;
  region?: string;
}

interface AuthenticationConfig {
  type: 'api-key' | 'oauth2' | 'mtls' | 'none';
  headerName?: string;
  tokenEndpoint?: string;        // For OAuth2
  scopes?: string[];
}

interface RateLimit {
  requestsPerMinute: number;
  requestsPerDay?: number;
  tokensPerMinute?: number;
}

interface VersionMetadata {
  changelog: string;
  breakingChanges: boolean;
  previousVersion?: string;
  aliases: string[];             // e.g., ['latest', 'stable']
  compatibility: CompatibilityInfo;
}

interface CompatibilityInfo {
  compatibleWith: string[];      // Compatible version ranges
  incompatibleWith: string[];
  migrationGuide?: string;       // URL to migration guide
}

type ModelStatus =
  | 'active'
  | 'deprecated'
  | 'retired'
  | 'experimental';

interface ModelDependency {
  dependencyId: string;          // Dependency model/pipeline ID
  dependencyType: 'model' | 'pipeline' | 'dataset' | 'config';
  versionConstraint: string;     // e.g., ">=2.0.0 <3.0.0"
  required: boolean;
}

interface PolicyAssociation {
  policyId: string;
  policyVersion: string;
  associatedAt: Date;
  associatedBy: string;
  exceptions: PolicyException[];
}

interface PolicyException {
  exceptionId: string;
  reason: string;
  approvedBy: string;
  approvedAt: Date;
  expiresAt?: Date;
}

type ComplianceStatus = {
  status: 'compliant' | 'non-compliant' | 'pending' | 'exempt' | 'unknown';
  lastCheckedAt: Date;
  violations: ComplianceViolation[];
  score?: number;                // 0-100 compliance score
};

interface ComplianceViolation {
  policyId: string;
  severity: 'critical' | 'high' | 'medium' | 'low';
  message: string;
  detectedAt: Date;
  remediation?: string;
}
```

### 5.2 Pipeline Configuration Schema

```typescript
interface PipelineConfiguration {
  // Identity
  id: string;
  name: string;
  version: SemanticVersion;

  // Definition
  dag: PipelineDAG;

  // Metadata
  description: string;
  tags: string[];
  owner: string;
  namespace: string;

  // Lifecycle
  status: PipelineStatus;

  // Configuration
  inputSchema: JSONSchema;
  outputSchema: JSONSchema;

  // Resources
  resources: ResourceRequirements;

  // Execution
  executionConfig: ExecutionConfig;

  // Relations
  dependencies: PipelineDependency[];

  // Compliance
  policyAssociations: PolicyAssociation[];
  complianceStatus: ComplianceStatus;

  // Timestamps
  createdAt: Date;
  updatedAt: Date;
}

interface PipelineDAG {
  nodes: PipelineNode[];
  edges: PipelineEdge[];
}

interface PipelineNode {
  id: string;
  type: 'model' | 'transform' | 'filter' | 'aggregate' | 'custom';
  config: NodeConfig;
  inputPorts: string[];
  outputPorts: string[];
}

type NodeConfig =
  | ModelNodeConfig
  | TransformNodeConfig
  | CustomNodeConfig;

interface ModelNodeConfig {
  modelId: string;               // Reference to registry model
  modelVersion: string;
  parameters: Record<string, unknown>;
  retry: RetryPolicy;
}

interface TransformNodeConfig {
  transformType: string;
  transformLogic: string;        // Code or reference
  language: 'javascript' | 'python' | 'sql';
}

interface CustomNodeConfig {
  handlerUri: string;
  configuration: Record<string, unknown>;
}

interface PipelineEdge {
  from: { nodeId: string; port: string };
  to: { nodeId: string; port: string };
  transform?: DataTransform;
}

interface DataTransform {
  type: 'map' | 'filter' | 'validate';
  logic: string;
}

type PipelineStatus =
  | 'draft'
  | 'active'
  | 'paused'
  | 'deprecated'
  | 'archived';

interface ResourceRequirements {
  cpu: string;                   // e.g., "500m", "2"
  memory: string;                // e.g., "512Mi", "2Gi"
  gpu?: GPURequirements;
  storage?: string;
}

interface GPURequirements {
  type: string;                  // e.g., "nvidia-tesla-v100"
  count: number;
  memory?: string;
}

interface ExecutionConfig {
  timeout: number;               // milliseconds
  retryPolicy: RetryPolicy;
  concurrency: number;
  priority: 'low' | 'normal' | 'high';
  environment: Record<string, string>;
}

interface RetryPolicy {
  maxAttempts: number;
  backoff: 'constant' | 'linear' | 'exponential';
  initialDelay: number;          // milliseconds
  maxDelay: number;
}

interface PipelineDependency {
  dependencyId: string;
  dependencyType: 'pipeline' | 'dataset' | 'config';
  versionConstraint: string;
}
```

### 5.3 Test Suite Schema

```typescript
interface TestSuite {
  // Identity
  id: string;
  name: string;
  version: string;

  // Target
  targetType: 'model' | 'pipeline';
  targetId: string;
  targetVersion: string;

  // Test Cases
  testCases: TestCase[];

  // Configuration
  config: TestSuiteConfig;

  // Metadata
  description: string;
  tags: string[];
  owner: string;

  // Timestamps
  createdAt: Date;
  updatedAt: Date;
}

interface TestCase {
  id: string;
  name: string;
  type: TestCaseType;

  // Test Definition
  input: unknown;
  expectedOutput?: unknown;
  assertions: Assertion[];

  // Configuration
  timeout?: number;
  retries?: number;

  // Metadata
  category: string;
  priority: 'critical' | 'high' | 'medium' | 'low';
  tags: string[];
}

type TestCaseType =
  | 'functional'     // Correctness tests
  | 'performance'    // Latency/throughput tests
  | 'compliance'     // Policy compliance tests
  | 'security'       // Security validation
  | 'integration'    // End-to-end tests
  | 'regression';    // Regression prevention

interface Assertion {
  type: AssertionType;
  config: AssertionConfig;
}

type AssertionType =
  | 'equals'
  | 'contains'
  | 'matches_regex'
  | 'less_than'
  | 'greater_than'
  | 'schema_valid'
  | 'custom';

type AssertionConfig = Record<string, unknown>;

interface TestSuiteConfig {
  parallel: boolean;
  stopOnFailure: boolean;
  reportFormat: 'json' | 'junit' | 'html';
  coverage: CoverageConfig;
}

interface CoverageConfig {
  enabled: boolean;
  threshold: number;             // Minimum coverage percentage
  includes: string[];
  excludes: string[];
}

interface TestResult {
  id: string;
  testSuiteId: string;
  testSuiteVersion: string;
  targetId: string;
  targetVersion: string;

  // Execution
  executedAt: Date;
  duration: number;              // milliseconds
  status: 'passed' | 'failed' | 'error' | 'skipped';

  // Results
  testCaseResults: TestCaseResult[];
  summary: TestSummary;

  // Performance
  performanceMetrics: PerformanceMetrics;

  // Artifacts
  artifacts: TestArtifact[];
}

interface TestCaseResult {
  testCaseId: string;
  status: 'passed' | 'failed' | 'error' | 'skipped';
  duration: number;
  actualOutput?: unknown;
  assertionResults: AssertionResult[];
  error?: ErrorInfo;
}

interface AssertionResult {
  assertionType: AssertionType;
  passed: boolean;
  message?: string;
  expected?: unknown;
  actual?: unknown;
}

interface TestSummary {
  total: number;
  passed: number;
  failed: number;
  errors: number;
  skipped: number;
  passRate: number;              // Percentage
}

interface PerformanceMetrics {
  averageLatency: number;
  p50Latency: number;
  p95Latency: number;
  p99Latency: number;
  throughput: number;            // ops/second
  errorRate: number;
}

interface TestArtifact {
  name: string;
  type: 'log' | 'screenshot' | 'report' | 'coverage' | 'trace';
  uri: string;
  size: number;
}

interface ErrorInfo {
  type: string;
  message: string;
  stack?: string;
  context?: Record<string, unknown>;
}
```

### 5.4 Lineage and Dependency Schema

```typescript
interface LineageNode {
  id: string;
  type: 'model' | 'pipeline' | 'dataset' | 'config' | 'test';
  name: string;
  version: string;
  metadata: Record<string, unknown>;

  // Temporal
  validFrom: Date;
  validTo?: Date;
}

interface LineageEdge {
  id: string;
  fromNode: string;              // Source node ID
  toNode: string;                // Target node ID
  relationshipType: RelationshipType;

  // Metadata
  metadata: Record<string, unknown>;

  // Temporal
  createdAt: Date;
  deletedAt?: Date;
}

type RelationshipType =
  | 'depends_on'      // Direct dependency
  | 'derived_from'    // Model derived from another
  | 'transforms'      // Pipeline transforms data
  | 'validates'       // Test validates model
  | 'configures'      // Config applies to model
  | 'replaces'        // Newer version replaces older
  | 'uses';           // General usage relationship

interface LineageGraph {
  nodes: LineageNode[];
  edges: LineageEdge[];
  metadata: GraphMetadata;
}

interface GraphMetadata {
  rootNode: string;
  depth: number;
  nodeCount: number;
  edgeCount: number;
  generatedAt: Date;
}

interface ImpactAnalysis {
  targetNode: string;
  impactedNodes: ImpactedNode[];
  depth: number;
  analysis: ImpactSummary;
}

interface ImpactedNode {
  nodeId: string;
  nodeName: string;
  nodeType: string;
  distance: number;              // Hops from target
  impactType: 'breaking' | 'non-breaking' | 'unknown';
  severity: 'critical' | 'high' | 'medium' | 'low';
  path: string[];                // Path from target to impacted node
}

interface ImpactSummary {
  totalImpacted: number;
  breakingChanges: number;
  criticalImpacts: number;
  recommendedActions: string[];
}
```

### 5.5 Audit Log Schema

```typescript
interface AuditLogEntry {
  // Identity
  id: string;
  eventId: string;               // Idempotency key

  // Event Details
  eventType: AuditEventType;
  eventCategory: 'read' | 'write' | 'delete' | 'admin';

  // Actor
  userId: string;
  userEmail?: string;
  serviceAccount?: string;
  ipAddress: string;
  userAgent?: string;

  // Resource
  resourceType: 'model' | 'pipeline' | 'test' | 'policy' | 'config';
  resourceId: string;
  resourceVersion?: string;
  namespace: string;

  // Action
  action: string;                // e.g., 'create', 'update', 'delete'

  // Changes
  before?: Record<string, unknown>;
  after?: Record<string, unknown>;
  diff?: JSONPatch[];

  // Context
  requestId: string;
  sessionId?: string;
  metadata: Record<string, unknown>;

  // Result
  status: 'success' | 'failure' | 'error';
  statusCode?: number;
  errorMessage?: string;

  // Timing
  timestamp: Date;
  duration?: number;             // milliseconds

  // Security
  authenticationMethod: 'api-key' | 'oauth2' | 'mtls';
  authorizationDecision: 'allowed' | 'denied';

  // Compliance
  complianceRelevant: boolean;
  retentionPeriod: number;       // days
}

type AuditEventType =
  | 'model.created'
  | 'model.updated'
  | 'model.deleted'
  | 'model.accessed'
  | 'pipeline.created'
  | 'pipeline.updated'
  | 'pipeline.executed'
  | 'test.executed'
  | 'policy.associated'
  | 'policy.violated'
  | 'compliance.checked'
  | 'permission.granted'
  | 'permission.denied'
  | 'config.updated'
  | 'admin.operation';

interface JSONPatch {
  op: 'add' | 'remove' | 'replace' | 'move' | 'copy' | 'test';
  path: string;
  from?: string;
  value?: unknown;
}
```

### 5.6 Extensibility Schema

```typescript
interface CustomMetadataSchema {
  // Schema Definition
  schemaId: string;
  schemaName: string;
  schemaVersion: string;
  namespace: string;

  // JSON Schema
  jsonSchema: JSONSchema;

  // Configuration
  applicableTo: ('model' | 'pipeline' | 'test')[];
  required: boolean;

  // Indexing
  indexableFields: IndexableField[];

  // Validation
  validationRules: ValidationRule[];

  // Metadata
  owner: string;
  createdAt: Date;
  updatedAt: Date;
}

interface IndexableField {
  fieldPath: string;             // JSON path to field
  indexType: 'btree' | 'hash' | 'fulltext' | 'spatial';
  unique: boolean;
}

interface ValidationRule {
  ruleType: 'regex' | 'range' | 'enum' | 'custom';
  config: Record<string, unknown>;
  errorMessage: string;
}

// JSON Schema type (simplified)
interface JSONSchema {
  type: 'object' | 'array' | 'string' | 'number' | 'boolean' | 'null';
  properties?: Record<string, JSONSchema>;
  items?: JSONSchema;
  required?: string[];
  enum?: unknown[];
  minimum?: number;
  maximum?: number;
  pattern?: string;
  additionalProperties?: boolean | JSONSchema;
}
```

---

## 6. Success Criteria

### 6.1 Functional Success Criteria

1. **Model Registration**
   - Successfully register 1000 unique models with complete metadata
   - Zero data loss or corruption during registration
   - 100% of registrations complete within SLA (<200ms p95)

2. **Query Performance**
   - 95% of queries return results in <50ms
   - Support 10,000 concurrent read queries without degradation
   - Zero query failures due to system errors

3. **Lineage Tracking**
   - Accurately track multi-level dependencies (depth >10)
   - Correctly identify all downstream impacts for updates
   - 100% accuracy in circular dependency detection

4. **Compliance Integration**
   - Real-time compliance status updates (<1 second lag)
   - 100% of policy violations detected and recorded
   - Zero false positives in compliance reporting

5. **Multi-Component Integration**
   - Successfully integrate with all 4 ecosystem components
   - <5 second end-to-end latency for cross-component workflows
   - 99.9% success rate for integration API calls

### 6.2 Non-Functional Success Criteria

1. **Scalability**
   - Scale to 1M+ models without performance degradation
   - Horizontal scaling adds capacity linearly
   - Query latency remains <100ms at 10M+ models

2. **Reliability**
   - Achieve 99.95% uptime over 30-day period
   - RTO <5 minutes for all failure scenarios
   - Zero data loss in disaster recovery tests

3. **Security**
   - Pass security audit with zero critical vulnerabilities
   - 100% of audit logs captured and retained
   - Zero unauthorized access attempts succeed

4. **Performance**
   - Sustain 1000+ writes/second per node
   - Handle 10,000+ concurrent operations
   - Maintain p95 latency SLOs under load

5. **Observability**
   - 100% of operations traced and logged
   - All critical metrics exported and monitored
   - <5 minute MTTD (Mean Time To Detect) for anomalies

### 6.3 User Experience Success Criteria

1. **Developer Productivity**
   - <5 minutes to register first model
   - <2 minutes to query and discover models
   - <10 minutes to set up lineage tracking for a pipeline

2. **Documentation**
   - 100% API coverage in documentation
   - <30 minutes for new users to complete quickstart
   - >90% user satisfaction with documentation

3. **Error Handling**
   - 100% of errors return actionable error messages
   - All failures include remediation guidance
   - <10% support ticket rate for common operations

### 6.4 Integration Success Criteria

1. **Memory-Graph Sync**
   - <1 second sync lag for lineage updates
   - 100% graph consistency with registry state
   - Support graph queries with <100ms latency

2. **Policy Engine Validation**
   - <5 seconds for policy validation workflows
   - 100% of policy changes reflected in registry
   - Zero stale compliance statuses

3. **Forge SDK Sync**
   - <1 hour propagation time for SDK updates
   - 100% of model schemas correctly exported
   - Zero breaking changes in SDK without versioning

4. **Dashboard Integration**
   - <200ms latency for dashboard queries
   - Real-time updates with <2 second lag
   - Support 1000+ concurrent dashboard users

### 6.5 Compliance and Governance Success Criteria

1. **Audit Trail**
   - 100% of operations logged to audit trail
   - Zero audit log data loss
   - 7-year retention with point-in-time recovery

2. **Policy Compliance**
   - 100% of models have associated policies
   - <1% false positive rate in compliance checks
   - 100% of violations trigger alerts

3. **Data Governance**
   - 100% of PII encrypted at rest and in transit
   - Pass compliance audits (SOC2, ISO27001, GDPR)
   - Zero data retention policy violations

---

## 7. Out of Scope

The following are explicitly out of scope for LLM-Registry:

1. **Model Training**: Registry does not train or fine-tune models
2. **Model Serving**: Registry does not serve inference requests
3. **Data Storage**: Registry stores metadata only, not model weights or datasets
4. **Policy Enforcement**: Registry tracks compliance but does not enforce policies (LLM-Policy-Engine responsibility)
5. **User Management**: Delegates to external identity provider
6. **Billing/Metering**: Does not handle usage-based billing
7. **Model Marketplace**: Does not provide model discovery marketplace (future consideration)

---

## 8. Assumptions and Constraints

### 8.1 Assumptions

1. All model providers expose stable, versioned APIs
2. Clients will use SDK or follow API best practices
3. Network latency between services <10ms within region
4. Identity provider (IdP) is highly available (99.99%+)
5. Storage layer (database) supports ACID transactions

### 8.2 Constraints

1. **Technology Stack**: Must use technologies compatible with Node.js/TypeScript ecosystem
2. **Cloud-Native**: Must support deployment on Kubernetes
3. **Database**: Must support relational database (PostgreSQL) and document store (MongoDB)
4. **Budget**: Infrastructure costs must remain <$X per month at 1M models
5. **Compliance**: Must comply with SOC2, GDPR, and HIPAA requirements
6. **Open Source**: Must use Apache 2.0 or MIT licensed dependencies only

### 8.3 Dependencies

1. **External Services**:
   - Identity Provider (OAuth2/OIDC)
   - Message Queue (Kafka or RabbitMQ)
   - Cache (Redis)
   - Search Engine (Elasticsearch/OpenSearch)
   - Object Storage (S3-compatible)

2. **Ecosystem Components**:
   - LLM-Memory-Graph (for lineage storage)
   - LLM-Policy-Engine (for compliance validation)
   - LLM-Forge (for SDK generation)
   - LLM-Governance-Dashboard (for visualization)

3. **Infrastructure**:
   - Kubernetes cluster (version 1.25+)
   - Load balancer with TLS termination
   - Distributed tracing backend (Jaeger/Tempo)
   - Metrics backend (Prometheus/Grafana)

---

## 9. Implementation Phases (High-Level)

**Phase 1: Core Registry (Weeks 1-4)**
- Model metadata CRUD operations
- Basic versioning
- Query API
- Authentication/authorization

**Phase 2: Pipeline & Test Suites (Weeks 5-8)**
- Pipeline configuration storage
- Test suite definitions
- DAG validation

**Phase 3: Lineage & Dependencies (Weeks 9-12)**
- Dependency graph construction
- Lineage tracking
- Impact analysis

**Phase 4: Integrations (Weeks 13-16)**
- Memory-Graph integration
- Policy Engine hooks
- Forge SDK sync
- Dashboard APIs

**Phase 5: Advanced Features (Weeks 17-20)**
- Custom metadata schemas
- Advanced query capabilities
- Performance optimizations
- Multi-region deployment

**Phase 6: Production Hardening (Weeks 21-24)**
- Security audit
- Load testing
- Disaster recovery testing
- Documentation completion

---

This specification provides a comprehensive blueprint for the LLM-Registry system, covering all functional, non-functional, and integration requirements necessary for successful implementation within the LLM ecosystem.
