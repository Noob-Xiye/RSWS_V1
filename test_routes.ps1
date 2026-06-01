# 测试路由匹配
$ErrorActionPreference = "Stop"

# 启动服务器（后台）
$serverProcess = Start-Process -FilePath "F:\GitRepo\RSWS_V1\target\debug\rsws.exe" `
    -WorkingDirectory "F:\GitRepo\RSWS_V1" `
    -Environment @{ RUST_LOG = "info" } `
    -PassThru `
    -WindowStyle Normal

Write-Host "Server started (PID: $($serverProcess.Id))"
Start-Sleep -Seconds 3

# 测试健康检查
Write-Host "`n=== Testing /health ==="
try {
    $health = Invoke-RestMethod -Uri "http://localhost:58080/health" -Method Get -TimeoutSec 5
    Write-Host "✅ Health check passed: $health" -ForegroundColor Green
} catch {
    Write-Host "❌ Health check failed: $_" -ForegroundColor Red
}

# 测试需要认证的路由（会返回 401，但证明路由存在）
Write-Host "`n=== Testing /api/v1/admin/email-configs (GET) ==="
try {
    $resp = Invoke-RestMethod -Uri "http://localhost:58080/api/v1/admin/email-configs" -Method Get -TimeoutSec 5
    Write-Host "✅ Route exists (unexpected success)" -ForegroundColor Green
} catch {
    if ($_.Exception.Response.StatusCode -eq 401) {
        Write-Host "✅ Route exists (401 Unauthorized - expected)" -ForegroundColor Green
    } else {
        Write-Host "❌ Route missing or error: $_" -ForegroundColor Red
    }
}

Write-Host "`n=== Testing /api/v1/admin/usdt-wallets (GET) ==="
try {
    $resp = Invoke-RestMethod -Uri "http://localhost:58080/api/v1/admin/usdt-wallets" -Method Get -TimeoutSec 5
    Write-Host "✅ Route exists (unexpected success)" -ForegroundColor Green
} catch {
    if ($_.Exception.Response.StatusCode -eq 401) {
        Write-Host "✅ Route exists (401 Unauthorized - expected)" -ForegroundColor Green
    } else {
        Write-Host "❌ Route missing or error: $_" -ForegroundColor Red
    }
}

Write-Host "`n=== All route tests completed ==="
Write-Host "Server running at PID: $($serverProcess.Id)"
Write-Host "Stop server with: Stop-Process -Id $($serverProcess.Id)"
