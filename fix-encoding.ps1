# UTF-16 到 UTF-8 无 BOM 编码转换脚本
# 用于修复 RSWS_V1 项目中的编码问题

param(
    [string]$ProjectPath = "F:\gitrepo\RSWS_V1",
    [switch]$WhatIf,
    [switch]$Backup
)

Write-Host "=== RSWS_V1 项目编码修复工具 ===" -ForegroundColor Cyan
Write-Host "项目路径: $ProjectPath" -ForegroundColor Gray

if ($WhatIf) {
    Write-Host "`n[预览模式] 不会实际修改文件`n" -ForegroundColor Yellow
}

# 需要转换的文件类型
$patterns = @('*.rs', '*.toml', '*.yml', '*.yaml', '*.md', '*.sql', '*.py', '*.json', '*.html', '*.css', '*.js', '*.ts', '*.env', '*.http', '*.lock', '*.txt', '*.log', '*.conf', '*.cfg', '*.ini')

$utf16Files = @()

# 扫描 UTF-16 文件
Write-Host "正在扫描 UTF-16 编码的文件..." -ForegroundColor Cyan

foreach ($pattern in $patterns) {
    $files = Get-ChildItem -Path $ProjectPath -Recurse -File -Filter $pattern -ErrorAction SilentlyContinue
    
    foreach ($file in $files) {
        try {
            $stream = [System.IO.File]::OpenRead($file.FullName)
            $buffer = New-Object byte[] 2
            $bytesRead = $stream.Read($buffer, 0, 2)
            $stream.Close()
            
            if ($bytesRead -ge 2) {
                if (($buffer[0] -eq 0xFF -and $buffer[1] -eq 0xFE) -or 
                    ($buffer[0] -eq 0xFE -and $buffer[1] -eq 0xFF)) {
                    $utf16Files += $file.FullName
                }
            }
        }
        catch {
            # 跳过无法读取的文件
        }
    }
}

if ($utf16Files.Count -eq 0) {
    Write-Host "`n✓ 未发现 UTF-16 编码的文件" -ForegroundColor Green
    exit 0
}

Write-Host "`n发现 $($utf16Files.Count) 个 UTF-16 编码的文件:`n" -ForegroundColor Yellow

$fixedCount = 0
$errorCount = 0

foreach ($file in $utf16Files) {
    $relativePath = $file.Substring($ProjectPath.Length + 1)
    Write-Host "  处理: $relativePath" -ForegroundColor Gray
    
    if (-not $WhatIf) {
        try {
            # 备份原文件（可选）
            if ($Backup) {
                $backupPath = "$file.backup"
                Copy-Item -Path $file -Destination $backupPath -Force
            }
            
            # 读取 UTF-16 文件内容
            $content = Get-Content -Path $file -Raw -Encoding Unicode
            
            # 转换为 UTF-8 无 BOM 并写回
            $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
            [System.IO.File]::WriteAllText($file, $content, $utf8NoBom)
            
            $fixedCount++
            Write-Host "    ✓ 已转换为 UTF-8 无 BOM" -ForegroundColor Green
        }
        catch {
            $errorCount++
            Write-Host "    ✗ 转换失败: $_" -ForegroundColor Red
        }
    } else {
        $fixedCount++
        Write-Host "    [预览] 将转换为 UTF-8 无 BOM" -ForegroundColor Yellow
    }
}

Write-Host "`n=== 转换完成 ===" -ForegroundColor Cyan
if (-not $WhatIf) {
    Write-Host "成功: $fixedCount 个文件" -ForegroundColor Green
    if ($errorCount -gt 0) {
        Write-Host "失败: $errorCount 个文件" -ForegroundColor Red
    }
} else {
    Write-Host "预览模式: 将转换 $fixedCount 个文件" -ForegroundColor Yellow
    Write-Host "`n运行时不加 -WhatIf 参数来执行实际转换" -ForegroundColor Gray
}

# 创建 .editorconfig 以统一编码规范
$editorconfigPath = Join-Path $ProjectPath ".editorconfig"
if (-not (Test-Path $editorconfigPath)) {
    Write-Host "`n建议创建 .editorconfig 文件以统一编码规范" -ForegroundColor Cyan
    Write-Host "运行脚本将自动创建" -ForegroundColor Gray
}

Write-Host "`n=== 下一步 ===" -ForegroundColor Cyan
Write-Host "1. 检查转换后的文件是否正常" -ForegroundColor Yellow
Write-Host "2. 编译项目确保没有编码相关问题" -ForegroundColor Yellow
Write-Host "3. 提交更改到 Git" -ForegroundColor Yellow
