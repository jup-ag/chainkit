"""
Some mutations need to happen post generation:

- ChainKit.swift needs a swiftlint flag to make the linter happy (lines are too long!)
- The `canImport` needs to disappear. Even though it works in other projects, it fails in ours
"""

def process_chainkit():
    found_can_import = False
    version = get_version()
    lines = [
        "// swiftlint:disable all\n",
        "// version: %s\n" %(version,)
    ]

    chain_kit = "platforms/ios/ChainKit/Sources/ChainKit/ChainKit.swift"
    for line in open(chain_kit, "r").readlines():
        # remove the `canImport` line, see above
        if line.find("canImport") > 0:
            found_can_import = True
            continue
        # Once the line was removed, remove the `endif` line
        if line.find("endif") > 0 and found_can_import:
            found_can_import = False
            continue
        lines.append(line)

    open(chain_kit, "w").write("".join(lines))

def get_version():
    import subprocess
    params = [
        "git", 
        "rev-parse", 
        "HEAD", 
    ]
    version = subprocess.run(params, capture_output=True).stdout.decode()
    return version.strip()

if __name__ == "__main__":
    process_chainkit()
