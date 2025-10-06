# Performance Optimization Guide

## Problem: "Too many open files (os error 24)"

This error occurs when the application exceeds the system's file descriptor limit, typically caused by:
- Too many database connections
- Connection leaks
- Insufficient system limits

## Implemented Solutions

### 1. Database Pool Configuration

**File**: `src/config/database.rs`
- Reduced `max_connections` from 50 to 20 (suitable for 2GB RAM)
- Increased `min_connections` to 5 for connection warmup
- Optimized timeouts: `acquire_timeout` 10s, `idle_timeout` 60s
- Enabled statement caching (128 capacity)
- Added connection health checks with `test_before_acquire`

### 2. Environment Variables

**File**: `.env`
```env
DB_MAX_CONNECTIONS=20
DB_MIN_CONNECTIONS=5
DB_ACQUIRE_TIMEOUT_SECS=10
DB_IDLE_TIMEOUT_SECS=60
DB_MAX_LIFETIME_SECS=3600
REDIS_MAX_CONNECTIONS=5
```

### 3. Graceful Shutdown

**File**: `src/main.rs`
- Added signal handling for SIGTERM and SIGINT
- Proper database pool closure on shutdown
- Prevents connection leaks during application restart

### 4. Connection Monitoring

**File**: `src/utils/connection_monitor.rs`
- Real-time monitoring of connection pool status
- Health checks every 30 seconds
- Alerts when pool is exhausted
- Logs active/idle connection counts

### 5. Query Optimization

**File**: `src/repositories/auth.rs`
- Optimized login query to select only required fields: `id`, `email`, `password`
- Reduced data transfer and memory usage

## System Tuning Recommendations

### Linux/Unix Systems

1. **Increase file descriptor limits**:
   ```bash
   # Temporary (current session)
   ulimit -n 65536
   
   # Permanent (add to /etc/security/limits.conf)
   * soft nofile 65536
   * hard nofile 65536
   ```

2. **PostgreSQL Configuration**:
   ```sql
   -- postgresql.conf
   max_connections = 100
   shared_buffers = 256MB
   effective_cache_size = 1GB
   work_mem = 4MB
   ```

### Windows Systems

1. **Increase handle limits** via Registry or Group Policy
2. **Monitor Resource Monitor** for handle usage
3. **Use Process Explorer** to track file handles per process

## Monitoring Commands

### Check current limits:
```bash
# Linux/Unix
ulimit -n
cat /proc/sys/fs/file-max

# Check current usage
lsof -p <pid> | wc -l
```

### PostgreSQL monitoring:
```sql
-- Check active connections
SELECT count(*) FROM pg_stat_activity;

-- Check connection by state
SELECT state, count(*) FROM pg_stat_activity GROUP BY state;

-- Check long-running queries
SELECT pid, now() - pg_stat_activity.query_start AS duration, query 
FROM pg_stat_activity 
WHERE (now() - pg_stat_activity.query_start) > interval '5 minutes';
```

## Performance Metrics

### Before Optimization:
- Failure rate: 77.21%
- Average response time: 4.13s
- P95 response time: 10.09s
- Throughput: 93 req/s

### Expected After Optimization:
- Failure rate: <5%
- Average response time: <500ms
- P95 response time: <1s
- Throughput: >500 req/s

## Load Testing

Run performance tests after optimization:

```bash
# Install k6
# Run load test
k6 run --vus 100 --duration 30s load-test.js
```

## Troubleshooting

### If errors persist:

1. **Check system resources**:
   ```bash
   htop
   free -h
   df -h
   ```

2. **Monitor database**:
   ```sql
   SELECT * FROM pg_stat_activity;
   ```

3. **Check application logs**:
   ```bash
   tail -f logs/app.log | grep -E "(error|ERROR|Too many)"
   ```

4. **Verify connection pool metrics** in application logs

## Additional Optimizations

### Future Improvements:
1. **Connection Pooling**: Consider PgBouncer for external connection pooling
2. **Caching**: Implement Redis caching for frequently accessed data
3. **Database Indexing**: Optimize queries with proper indexes
4. **Horizontal Scaling**: Load balancer with multiple application instances
5. **Database Replication**: Read replicas for read-heavy workloads

### Code-level Optimizations:
1. **Async Processing**: Use background jobs for heavy operations
2. **Batch Operations**: Group multiple database operations
3. **Connection Reuse**: Ensure proper connection lifecycle management
4. **Memory Management**: Profile and optimize memory usage