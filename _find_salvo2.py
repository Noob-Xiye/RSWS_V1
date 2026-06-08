import re
with open(r'F:\Gitrepo\rsws_v1\Cargo.lock') as f:
    text = f.read()
packages = re.findall(r'name = "([^"]+)"\nversion = "([^"]+)"', text)
for name, ver in packages:
    if 'salvo' in name:
        print(name, ver)
