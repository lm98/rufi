#!/bin/bash

HOOK_NAMES="commit-msg"
GIT_HOOK_DIR=../.git/hooks
LOCAL_HOOK_DIR=./

echo "Installing git-hooks in project"

for HOOK in $HOOK_NAMES; do
    	if [ -f $LOCAL_HOOK_DIR/$HOOK ]; then
		echo "Installing hook $HOOK"
      		if [ ! -h $GIT_HOOK_DIR/$HOOK -a -x $GIT_HOOK_DIR/$HOOK ]; then
			echo "Hook $HOOK already exists, skipping."
		else
			cp $LOCAL_HOOK_DIR/$HOOK $GIT_HOOK_DIR/$HOOK
			chmod 744 $GIT_HOOK_DIR/$HOOK
			echo "Hook $HOOK installed"
		fi
	fi
done