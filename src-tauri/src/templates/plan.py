import sys

def main():
    print("# Project Plan")
    print("Creating Gantt chart structure...")
    print("```mermaid")
    print("gantt")
    print("    title Project Schedule")
    print("    section Design")
    print("    UI Mockups :a1, 2024-01-01, 30d")
    print("    section Development")
    print("    Backend API :after a1, 20d")
    print("```")

if __name__ == "__main__":
    main()
