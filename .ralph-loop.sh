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
