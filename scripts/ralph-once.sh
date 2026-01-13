#!/bin/bash
set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <plan-name>"
    echo "Example: $0 my-feature  (uses plans/my-feature.md)"
    exit 1
fi

GIT_ROOT=$(git rev-parse --show-toplevel)
PLAN_NAME="$1"
PLAN_FILE="plans/${PLAN_NAME}.md"

if [ ! -f "$GIT_ROOT/$PLAN_FILE" ]; then
    echo "Error: Plan file '$PLAN_FILE' not found"
    exit 1
fi

cd "$GIT_ROOT"

echo "Running single iteration for $PLAN_NAME..."
claude --print --permission-mode bypassPermissions -p "@$PLAN_FILE @progress.txt \
1. Find the highest-priority task and implement it. \
2. Run your tests and type checks. \
3. Update the plan with what was done. \
4. Append your progress to progress.txt. \
5. Commit your changes. \
ONLY WORK ON A SINGLE TASK. \
If the plan is complete, output <promise>COMPLETE</promise>."
