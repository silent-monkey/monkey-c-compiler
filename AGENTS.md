# General Requirements

1. The agent shall not modify the file "AGENTS.md".
2. The agent shall not modify the directory "TASK".
3. The agent should edit the file ".gitignore" to exclude build artifacts from tracking.
5. The agent shall commit its work after finishing the task.

# Environment Management

1. The agent shall not attempt to get root privilege. The environment is configured to reject such requests.
2. The agent shall INTERRUPT the task when an essential step of the task does require root privilege.
3. The agent shall not install software, either by invoking the system package manager,
   or by downloading and excuting installation script directly.
4. The agent shall INTERRUPT the task when a tool/software/dependency is required but not installed.
   A required tool may be a tool explicily stated in the task description, or a tool that is essential for the task.
5. Generally the user is willing to setup the environment suitable for agents.
   When the agent finds the environment indeed has problems, prefer to INTERRUPT the task and report the problem to the user, than divert the task and put (useless in the long run) effort to finding workarounds.

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
