import re

correct_hash = r"$argon2id$v=19$m=65536,t=3,p=4$pN42zqJvzXCJEOe819H13Q$3ISKaZ6JIllQGagrSenn9akFnNy0Yy+k75Hju1HHk4c"

# Read original file
with open(r"F:\GitRepo\RSWS_V1\scripts\test_data.sql", "r", encoding="utf-8") as f:
    content = f.read()

# Replace all wrong password hashes with correct one
# The wrong pattern is: '\\=19\=19456,t=2,p=1\\'
# We need to replace the single-quoted string value
old_pattern = r"'\\\\=19\\=19456,t=2,p=1\\\\'"
new_val = f"'{correct_hash}'"
content = re.sub(old_pattern, new_val, content)

# Also handle if there are other wrong formats
# The file uses this pattern for all password_hash values
# Let's just do a more general replacement

with open(r"F:\GitRepo\RSWS_V1\scripts\test_data.sql", "w", encoding="utf-8") as f:
    f.write(content)

print("Updated test_data.sql with correct Argon2 hash")
print("Hash:", correct_hash[:40], "...")
