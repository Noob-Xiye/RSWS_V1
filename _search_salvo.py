import re
with open(r'F:\Gitrepo\rsws_v1\Cargo.lock') as f:
    text = f.read()

# Find salvo_extra details
idx = text.find('name = "salvo_extra"')
if idx >= 0:
    section = text[idx:idx+2000]
    print("salvo_extra:")
    print(section[:500])
else:
    print("salvo_extra not found")

# Check salvo_core
idx2 = text.find('name = "salvo_core"')
if idx2 >= 0:
    section2 = text[idx2:idx2+2000]
    print("\nsalvo_core deps:")
    # find deps
    dep_start = section2.find('dependencies = [')
    if dep_start >= 0:
        dep_end = section2.find(']', dep_start)
        print(section2[dep_start:dep_end+1])
