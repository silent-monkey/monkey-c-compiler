# General Requirements

1. The agent shall not modify the file "AGENTS.md".
2. The agent shall not modify the directory "TASK".
3. The agent should edit the file ".gitignore" to exclude build artifacts from tracking.
4. The agent shall commit its work after finishing the task.

# Git Commitment Guideline

The agent may commit its work in multiple commits.
The commit policy is up to the agent.
The agent may commit immeidately after one feature is done.
The agent may also commit after the whole task is finished in one or multiple commits.

The agent should write git commit message as following:
```
[vibe] Short imperative summary under 50 characters

The body of the message starts here after a blank line.
Wrap lines cleanly at 72 characters.

Assisted-by: AGENT_NAME:MODEL_VERSION
```

1. The short summary shall be prefixed by "[vibe] ".
2. The long message body may be omitted if the short summary suffices for small changes.
