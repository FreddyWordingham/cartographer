import os

from directory_tree import DisplayTree


def print_tree(dir):
    print("Directory tree:")
    print("```")
    DisplayTree(dir)
    print("```")
    print("\n")


def print_files(dir):
    # Recursively search for all files in the src directory
    for root, _dirs, files in os.walk(dir):
        for file in files:
            print(f"{os.path.join(root, file)}:")
            print("```rust")
            with open(os.path.join(root, file), "r") as f:
                print(f.read())
            print("```\n")



if __name__ == "__main__":
    print_tree("src")
    print_files("src")
