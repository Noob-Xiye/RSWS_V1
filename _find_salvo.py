import re
with open(r'F:\Gitrepo\rsws_v1\Cargo.lock') as f:
    text = f.read()
m = re.search(r'name = "salvo"\nversion = "([^"]+)"', text)
print(m.group(0) if m else 'not found')
