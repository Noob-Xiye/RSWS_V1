import argon2
h = argon2.PasswordHasher()
pwd_hash = h.hash("Admin123")
sql = f"UPDATE admins SET password_hash = '{pwd_hash}' WHERE email = 'admin@rsws.com';\n"
with open("F:/GitRepo/RSWS_V1/scripts/fix_admin_pwd.sql", "w", encoding="utf-8") as f:
    f.write(sql)
print(f"OK - {len(sql)} bytes")
print(f"Hash: {pwd_hash}")
