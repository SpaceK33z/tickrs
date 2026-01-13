#!/bin/bash
set -e

if [ -z "$1" ] || [ -z "$2" ]; then
    echo "Usage: $0 <plan-name> <iterations>"
    echo "Example: $0 my-feature 10  (uses plans/my-feature.md)"
    exit 1
fi

GIT_ROOT=$(git rev-parse --show-toplevel)
PLAN_NAME="$1"
PLAN_FILE="plans/${PLAN_NAME}.md"
SESSION="ralph-$PLAN_NAME"
ITERATIONS=$2

if [ ! -f "$GIT_ROOT/$PLAN_FILE" ]; then
    echo "Error: Plan file '$PLAN_FILE' not found"
    exit 1
fi

# Kill existing session if present
tmux kill-session -t "$SESSION" 2>/dev/null || true

echo "Starting $SESSION AFK session ($ITERATIONS iterations)..."
tmux new-session -d -s "$SESSION" -c "$GIT_ROOT"

# Create the loop script
cat > "$GIT_ROOT/.ralph-loop.sh" << 'SCRIPT'
#!/bin/bash
set -e
PLAN_FILE="$1"
ITERATIONS="$2"

for ((i=1; i<=ITERATIONS; i++)); do
    echo "=== Iteration $i of $ITERATIONS ==="
    result=$(claude --permission-mode acceptEdits -p "@$PLAN_FILE @progress.txt \
    1. Find the highest-priority task and implement it. \
    2. Run your tests and type checks. \
    3. Update the plan with what was done. \
    4. Append your progress to progress.txt. \
    5. Commit your changes. \
    ONLY WORK ON A SINGLE TASK. \
    If the plan is complete, output <promise>COMPLETE</promise>.")

    echo "$result"

    if [[ "$result" == *"<promise>COMPLETE</promise>"* ]]; then
        echo "Plan complete after $i iterations."
        exit 0
    fi
done
echo "Completed $ITERATIONS iterations."
SCRIPT
chmod +x "$GIT_ROOT/.ralph-loop.sh"

tmux send-keys -t "$SESSION" "bash .ralph-loop.sh $PLAN_FILE $ITERATIONS" Enter

echo "Running in tmux session '$SESSION'"
echo "Commands:"
echo "  tmux attach -t $SESSION    # Watch progress"
echo "  tmux kill-session -t $SESSION  # Stop"
