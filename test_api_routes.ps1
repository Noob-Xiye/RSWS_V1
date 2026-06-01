# 测试所有前端 API 路由
$baseUrl = "http://localhost:58080/api/v1"
$results = @()

Write-Host "=== 启动服务器 ===" -ForegroundColor Cyan
$env:RUST_LOG = "info"
$server = Start-Process -FilePath "F:\GitRepo\RSWS_V1\target\debug\rsws.exe" `
    -WorkingDirectory "F:\GitRepo\RSWS_V1" `
    -PassThru `
    -WindowStyle Hidden

Start-Sleep -Seconds 3
Write-Host "服务器已启动 (PID: $($server.Id))`n" -ForegroundColor Green

# 测试路由列表（从前端 API 文件提取）
$routes = @(
    @{ Method = "GET"; Path = "/health"; ExpectAuth = $false },
    @{ Method = "GET"; Path = "/admin/email-configs"; ExpectAuth = $true },
    @{ Method = "PUT"; Path = "/admin/email-configs"; ExpectAuth = $true },
    @{ Method = "GET"; Path = "/admin/usdt-wallets"; ExpectAuth = $true },
    @{ Method = "PUT"; Path = "/admin/usdt-wallets/ethereum"; ExpectAuth = $true },
    @{ Method = "GET"; Path = "/admin/dashboard/stats"; ExpectAuth = $true },
    @{ Method = "GET"; Path = "/admin/categories"; ExpectAuth = $true },
    @{ Method = "POST"; Path = "/admin/categories"; ExpectAuth = $true },
    @{ Method = "PUT"; Path = "/admin/categories/123"; ExpectAuth = $true }
)

Write-Host "=== 测试路由 ===" -ForegroundColor Cyan
foreach ($route in $routes) {
    $url = $baseUrl + $route.Path
    try {
        $resp = Invoke-WebRequest -Uri $url -Method $route.Method -TimeoutSec 5 -UseBasicParsing
        $status = $resp.StatusCode
        $statusColor = if ($status -eq 200) { "Green" } elseif ($status -eq 401) { "Yellow" } else { "Red" }
        Write-Host "✅ $($route.Method) $($route.Path) → $status" -ForegroundColor $statusColor
        $results += [PSCustomObject]@{ Method = $route.Method; Path = $route.Path; Status = $status; Error = "" }
    } catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        if ($statusCode -eq 401 -and $route.ExpectAuth) {
            Write-Host "✅ $($route.Method) $($route.Path) → 401 (需要认证)" -ForegroundColor Yellow
            $results += [PSCustomObject]@{ Method = $route.Method; Path = $route.Path; Status = 401; Error = "需要认证" }
        } elseif ($statusCode -eq 404) {
            Write-Host "❌ $($route.Method) $($route.Path) → 404 NOT FOUND" -ForegroundColor Red
            $results += [PSCustomObject]@{ Method = $route.Method; Path = $route.Path; Status = 404; Error = "路由不存在" }
        } else {
            Write-Host "⚠️  $($route.Method) $($route.Path) → $statusCode" -ForegroundColor Magenta
            $results += [PSCustomObject]@{ Method = $route.Method; Path = $route.Path; Status = $statusCode; Error = $_.Exception.Message }
        }
    }
}

Write-Host "`n=== 测试结果汇总 ===" -ForegroundColor Cyan
$results | Format-Table -AutoSize

$notFound = $results | Where-Object { $_.Status -eq 404 }
if ($notFound) {
    Write-Host "`n❌ 发现 $($notFound.Count) 个路由缺失：" -ForegroundColor Red
    $notFound | ForEach-Object { Write-Host "   $($_.Method) $($_.Path)" -ForegroundColor Red }
} else {
    Write-Host "`n✅ 所有路由都存在！" -ForegroundColor Green
}

Write-Host "`n=== 停止服务器 ===" -ForegroundColor Cyan
Stop-Process -Id $server.Id -Force
Write-Host "服务器已停止" -ForegroundColor Green
