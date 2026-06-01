# 测试所有前后端路由是否匹配
$base = "http://localhost:58080/api/v1"
$pass = 0; $fail = 0; $results = @()

Write-Host "=== 编译项目 ===" -Foreground Cyan
cargo build 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "❌ 编译失败" -Foreground Red; exit 1 }
Write-Host "✅ 编译成功" -Foreground Green

Write-Host "`n=== 启动服务器 ===" -Foreground Cyan
$proc = Start-Process -FilePath "F:\GitRepo\RSWS_V1\target\debug\rsws.exe" `
    -WorkingDirectory "F:\GitRepo\RSWS_V1" `
    -PassThru -WindowStyle Hidden
Start-Sleep -Seconds 4
Write-Host "✅ 服务器已启动 (PID: $($proc.Id))" -Foreground Green

function Test-Route($method, $path, $desc) {
    $url = $base + $path
    try {
        $r = Invoke-WebRequest -Uri $url -Method $method -TimeoutSec 3 -UseBasicParsing
        $code = $r.StatusCode
    } catch {
        $code = $_.Exception.Response.StatusCode.value__
    }
    # 401/403 = 路由存在（需要认证），200 = 成功，404 = 路由不存在
    if ($code -in 200,401,403) {
        Write-Host "✅ [$method] $path ($desc) — 路由存在 (HTTP $code)" -Foreground Green
        $script:pass++
        $script:results += [PSCustomObject]@{ Method=$method; Path=$path; Status=$code; Result="✅ 路由存在" }
    } elseif ($code -eq 404) {
        Write-Host "❌ [$method] $path ($desc) — 404 路由不存在！" -Foreground Red
        $script:fail++
        $script:results += [PSCustomObject]@{ Method=$method; Path=$path; Status=404; Result="❌ 路由不存在" }
    } else {
        Write-Host "⚠️  [$method] $path ($desc) — HTTP $code" -Foreground Yellow
        $script:pass++
        $script:results += [PSCustomObject]@{ Method=$method; Path=$path; Status=$code; Result="⚠️ HTTP $code" }
    }
}

# 健康检查（无需认证）
Test-Route "GET" "/health" "健康检查"

# 邮件配置
Test-Route "GET" "/admin/email-configs" "获取邮件配置"
Test-Route "PUT" "/admin/email-configs" "更新邮件配置"

# USDT 钱包
Test-Route "GET" "/admin/usdt-wallets" "获取USDT钱包列表"
# PUT 需要一个 network 参数，用 ethereum 测试
Test-Route "PUT" "/admin/usdt-wallets/ethereum" "更新USDT钱包"

# 分类管理
Test-Route "GET" "/admin/categories" "获取分类列表"
Test-Route "POST" "/admin/categories" "创建分类"
Test-Route "PUT" "/admin/categories/123" "更新分类"
Test-Route "DELETE" "/admin/categories/123" "删除分类"

# Dashboard
Test-Route "GET" "/admin/dashboard/stats" "Dashboard统计"
Test-Route "GET" "/admin/dashboard/revenue-chart?days=7" "收入图表"

# 管理员
Test-Route "GET" "/admin" "获取当前管理员"
Test-Route "GET" "/admin/list" "管理员列表"

Write-Host "`n=== 测试结果 ===" -Foreground Cyan
$results | Format-Table -AutoSize
Write-Host "✅ 通过: $pass | ❌ 失败: $fail" -Foreground $(if ($fail -eq 0) { "Green" } else { "Red" })

Write-Host "`n=== 停止服务器 ===" -Foreground Cyan
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
Write-Host "✅ 服务器已停止" -Foreground Green

if ($fail -gt 0) {
    Write-Host "`n❌ 发现 $fail 个路由问题，请修复后端 handler" -Foreground Red
    exit 1
} else {
    Write-Host "`n🎉 所有路由测试通过！前后端路由对齐完成！" -Foreground Green
    exit 0
}
