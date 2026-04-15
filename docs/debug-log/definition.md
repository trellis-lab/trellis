# Definition of a debug log support

## Problem

The port assignment and edge routing will be reviewed by both humans and AI assistants.
Currently it's hard to find out why a given port has been selected or why edge routed to a certain direction.
I want to have machine readable file which contains all important steps and details which can help to understand the decisions made the trellis core.

## Requirements

* The log file must be saved next to the rendered image by default.
* The location of the debug log file can be overridden at launch time.
* The log format must be structured and informative for an AI assistant.
* The debug log generation must not be in the released package. It will be used only in development and debugging sessions.
* There must be a new claude code command which can understand the debug log file and runs an analysis to answer questions of the user.
* The README.md file is updated by the latest command arguments.
* The help string should not have any reference of the debuglog. I want to keep it hidden from the customers due to security and IP protection.
