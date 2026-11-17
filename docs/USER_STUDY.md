<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# User study protocol: can an animator get real work done in OSATURA?

OSATURA has no external users yet (see [LIMITATIONS.md](LIMITATIONS.md), L-11). Its tests show that the engine does what its author
meant; they do not show that a working animator can use it. This protocol is the smallest study that would show it, and it is written
so that it can fail: the success criteria below are fixed **before** the first session.

This is a protocol, not a result. No session has been run.

## Who

Five animators or riggers who already use a rigging tool (Spine, Moho, Live2D, Rive, Harmony, Blender, Godot's 2D skeletons or similar),
at least two of them professionally. Not friends of the author if it can be avoided, and not OSATURA contributors. Each is told that the
tool, not they, is being tested, and may stop at any time.

## What each person does (90 minutes, one person at a time, screen shared)

The participant uses the studio (`osatura studio`) on a prepared project; the facilitator says nothing except what is written here.

| Task | Prepared material | Done when |
|------|-------------------|-----------|
| T1 Open and look around | the demo project | the participant has found the bones list, the timeline and the Layers panel, in their own words |
| T2 Rig a character | a PNG character in separate layers (ORA or PSD, imported with `import-ora` or `import-psd`) | two arms and a head move from bones; the pose at frame 12 is what the facilitator asked for |
| T3 Fix a deformation | a mesh with a bad weight at the elbow | the elbow bends cleanly when rotated 90 degrees, using the weight-painting section |
| T4 Animate a loop | the rigged character | a 24-frame walk or wave loop that plays without a jump at the loop point |
| T5 Get it out | the finished loop | the file is in the format the participant says they would use (GIF, sprite sheet, Lottie, HTML) and opens elsewhere |

The same tasks are then described to the participant for their own tool, and they estimate how long each would take there. The estimate is
data, not proof.

## What is recorded

* **Time** per task, from the facilitator's stopwatch, and whether the task was completed without the facilitator touching the keyboard.
* **Blockers**: every moment the participant stopped for more than 30 seconds, said "where is" or "how do I", or undid three times in a row.
* **Questionnaire** after the session: the ten questions of the System Usability Scale (SUS), scored 0 to 100, and three free-text
  questions: what would make you stop using it, what would make you recommend it, what is missing.
* **Screen recording** only if the participant agrees, kept off any public place.

No names, email addresses or employers are recorded with the data: participants get a number. The recording and the number-to-person list
are kept apart and deleted six months after the report is published.

## Success criteria (fixed now)

The study **supports** OSATURA as a tool for animators if all of these hold:

1. At least 4 of 5 complete T2, T4 and T5 without the facilitator touching the keyboard.
2. At least 3 of 5 complete T3, the weight-painting task.
3. The median SUS score is 68 or higher (the commonly quoted "average" point; it is a benchmark, not a verdict).
4. No single blocker is reported by 3 or more participants.

If criterion 1 or 2 fails, the report says which task and why, and the next work is that task, not new features. If criterion 4 fails, that blocker is
the next change. A study that passes on 3 of 4 is reported as failed on the fourth.

## What this cannot show

Five people cannot estimate how many animators will like the tool, and a session in a quiet room is not a production. A pass means "no
showstopper found at this size", not "ready". A fail is informative with five people; a pass is only encouraging.

## Report

Published as a document in the repository: the table of times, the blockers with counts, the SUS scores, the free-text answers
unedited except for removing anything that identifies a person, and the decision for each criterion. Raw numbers go in `docs/user-study/`
as CSV with the columns `participant, task, minutes, completed_unaided, blockers`.
