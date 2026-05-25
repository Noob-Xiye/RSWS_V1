# Fix: SERIAL -> BIGSERIAL for all primary keys, and matching foreign keys
path = r"F:\GitRepo\RSWS_V1\migrations\20260525000000_initial_schema.sql"
with open(path, "r", encoding="utf-8") as f:
    content = f.read()

# Fix 1: admins.id SERIAL -> BIGSERIAL
content = content.replace("    id SERIAL PRIMARY KEY,\n    email VARCHAR(255) NOT NULL UNIQUE,\n    password_hash VARCHAR(255) NOT NULL,\n    username VARCHAR(100) NOT NULL,\n    avatar_url VARCHAR(500),\n    is_active BOOLEAN DEFAULT true,\n    role VARCHAR(50) DEFAULT 'admin',",
                          "    id BIGSERIAL PRIMARY KEY,\n    email VARCHAR(255) NOT NULL UNIQUE,\n    password_hash VARCHAR(255) NOT NULL,\n    username VARCHAR(100) NOT NULL,\n    avatar_url VARCHAR(500),\n    is_active BOOLEAN DEFAULT true,\n    role VARCHAR(50) DEFAULT 'admin',")

# Fix 2: categories.id SERIAL -> BIGSERIAL  
content = content.replace("    id SERIAL PRIMARY KEY,\n    name VARCHAR(100) NOT NULL,\n    slug VARCHAR(100) NOT NULL UNIQUE,",
                          "    id BIGSERIAL PRIMARY KEY,\n    name VARCHAR(100) NOT NULL,\n    slug VARCHAR(100) NOT NULL UNIQUE,")

# Fix 3: user_payment_configs.id SERIAL -> BIGSERIAL
content = content.replace("    id SERIAL PRIMARY KEY,\n    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,",
                          "    id BIGSERIAL PRIMARY KEY,\n    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,")

# Fix 4: system_configs.id SERIAL -> BIGSERIAL
content = content.replace("    id SERIAL PRIMARY KEY,\n    config_key VARCHAR(100) NOT NULL UNIQUE,",
                          "    id BIGSERIAL PRIMARY KEY,\n    config_key VARCHAR(100) NOT NULL UNIQUE,")

# Fix 5: paypal_configs.id SERIAL -> BIGSERIAL
content = content.replace("    id SERIAL PRIMARY KEY,\n    client_id VARCHAR(255) NOT NULL,",
                          "    id BIGSERIAL PRIMARY KEY,\n    client_id VARCHAR(255) NOT NULL,")

# Fix 6: menu_items.id SERIAL -> BIGSERIAL
content = content.replace("    id SERIAL PRIMARY KEY,\n    parent_id INTEGER REFERENCES menu_items(id) ON DELETE CASCADE,",
                          "    id BIGSERIAL PRIMARY KEY,\n    parent_id BIGINT REFERENCES menu_items(id) ON DELETE CASCADE,")

# Fix 7: commission_rules.id SERIAL -> BIGSERIAL
content = content.replace("    id SERIAL PRIMARY KEY,\n    name VARCHAR(100) NOT NULL,\n    rule_type VARCHAR(50) NOT NULL,",
                          "    id BIGSERIAL PRIMARY KEY,\n    name VARCHAR(100) NOT NULL,\n    rule_type VARCHAR(50) NOT NULL,")

# Fix 8: admin_operation_logs.admin_id INTEGER -> BIGINT
content = content.replace("    admin_id INTEGER NOT NULL REFERENCES admins(id) ON DELETE CASCADE,",
                          "    admin_id BIGINT NOT NULL REFERENCES admins(id) ON DELETE CASCADE,")

# Fix 9: categories.parent_id INTEGER -> BIGINT (self-referencing)
content = content.replace("    parent_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,",
                          "    parent_id BIGINT REFERENCES categories(id) ON DELETE SET NULL,")

# Fix 10: commission_records.rule_id INTEGER -> BIGINT
content = content.replace("    rule_id INTEGER REFERENCES commission_rules(id) ON DELETE SET NULL,",
                          "    rule_id BIGINT REFERENCES commission_rules(id) ON DELETE SET NULL,")

with open(path, "w", encoding="utf-8") as f:
    f.write(content)

# Verify
with open(path, "r", encoding="utf-8") as f:
    content = f.read()

# Count remaining SERIAL
import re
sERIAL_lines = [(i+1, line) for i, line in enumerate(content.splitlines()) if 'SERIAL' in line]
BIGSERIAL_lines = [(i+1, line) for i, line in enumerate(content.splitlines()) if 'BIGSERIAL' in line]
print(f"BIGSERIAL count: {len(BIGSERIAL_lines)}")
print(f"Remaining SERIAL count: {len(sERIAL_lines)}")
if sERIAL_lines:
    print("WARNING - remaining SERIAL:")
    for ln, l in sERIAL_lines:
        print(f"  line {ln}: {l.strip()}")
else:
    print("All SERIAL fixed!")
