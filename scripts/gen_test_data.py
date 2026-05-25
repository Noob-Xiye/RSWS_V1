import os

# Argon2 hash for "Admin123"
# Generated with: python -c "from argon2 import PasswordHasher; print(PasswordHasher().hash('Admin123'))"
# Correct format: $argon2id$v=19$m=65536,t=3,p=4$<salt>$<hash>
# For simplicity, using a known working hash
password_hash = "$argon2id$v=19$m=65536,t=3,p=4$pN42zqJvzXCJEOe819H13Q$3ISKaZ6JIllQGagrSenn9akFnNy0Yy+k75Hju1HHk4c"

lines = []
lines.append("-- =======================================")
lines.append("-- RSWS 测试数据注入脚本（修正版）")
lines.append("-- =======================================")
lines.append("")
lines.append("-- ========== 管理员账号 ==========")
lines.append("INSERT INTO admins (id, email, password_hash, username, is_active, role, permissions, created_at, updated_at)")
lines.append(f"VALUES")
lines.append(f"  (1, 'admin@rsws.com',")
lines.append(f"   '{password_hash}',")
lines.append(f"   '超级管理员', true, 'super_admin', '[\"*\"]', NOW(), NOW())")
lines.append(f"ON CONFLICT (id) DO NOTHING;")
lines.append("")
lines.append("SELECT setval('admins_id_seq', COALESCE((SELECT MAX(id) FROM admins), 0) + 1, false);")
lines.append("")

sql = "\n".join(lines) + "\n"

out = r"F:\GitRepo\RSWS_V1\scripts\test_data_fixed.sql"
with open(out, "w", encoding="utf-8") as f:
    f.write(sql)

print(f"Wrote {len(sql)} chars -> test_data_fixed.sql")
print("Password hash:", password_hash[:60], "...")
