import subprocess

sqls = [
    "INSERT INTO system_logs (id, log_level, module, message, context, admin_id, ip_address, request_id) VALUES (9000001, 'INFO', 'auth', '管理员登录成功', '{\"email\": \"admin@rsws.com\"}'::jsonb, 1, '127.0.0.1', 'req-001')",
    "INSERT INTO system_logs (id, log_level, module, message, context, admin_id, ip_address, request_id) VALUES (9000002, 'INFO', 'resource', '资源创建成功', '{\"resource_id\": 1}'::jsonb, 1, '127.0.0.1', 'req-002')",
    "INSERT INTO system_logs (id, log_level, module, message, context, ip_address, request_id) VALUES (9000003, 'WARN', 'payment', '支付回调处理延迟', '{\"order_id\": 100, \"delay_ms\": 5000}'::jsonb, '203.0.113.5', 'req-003')",
    "INSERT INTO system_logs (id, log_level, module, message, context, admin_id, ip_address, request_id) VALUES (9000004, 'INFO', 'admin', '管理员配置更新', '{\"config_key\": \"log.level\"}'::jsonb, 1, '127.0.0.1', 'req-004')",
    "INSERT INTO system_logs (id, log_level, module, message, context, ip_address, request_id) VALUES (9000005, 'ERROR', 'auth', '登录失败: 密码错误', '{\"email\": \"admin@rsws.com\", \"attempts\": 3}'::jsonb, '198.51.100.42', 'req-005')",
]

for sql in sqls:
    r = subprocess.run(['docker', 'exec', 'rsws-postgres', 'psql', '-U', 'rsws', '-d', 'rsws', '-c', sql], capture_output=True, text=True)
    print(r.stdout.strip() or r.stderr.strip())
