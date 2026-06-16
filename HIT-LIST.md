# HIT LIST

Things that we should research, explore, and shape as improvements to harness-kit (or thoughtfully reject).

## Exa Agent

https://exa.ai/blog/exa-agent

Seems powerful. Might come for free given the current articulation of our research skill. Might need particular encouragement or something.

## Ponytail skill

https://github.com/DietrichGebert/ponytail

Might help us mitigate overengineering and keep things properly simple

## Top Agent Skills by GitHub Stars

Top 10 Agent Skills by GitHub Stars
The agent skills market has a clear signal:
Small, sharp workflows are winning.
I refreshed the top 10 most-starred agent skills on GitHub:
228,740 stars
Agentic skills framework and dev methodology.
https://github.com/obra/superpowers
151,088 stars
Official public Agent Skills repo from Anthropic.
https://github.com/anthropics/skills
130,016 stars
Real-world skill setup from Matt Pocock.
https://github.com/mattpocock/skills
110,407 stars
Claude Code setup for exec, design, engineering, docs, QA.
https://github.com/garrytan/gstack
92,040 stars
Design intelligence for better UI/UX.
https://github.com/nextlevelbuilder/ui-ux-pro-max-skill
60,442 stars
Turns code into an interactive knowledge graph.
https://github.com/Egonex-AI/Understand-Anything
60,265 stars
Production-grade engineering skills for coding agents.
https://github.com/addyosmani/agent-skills
53,903 stars
AI-powered job search system using Claude Code skill modes.
https://github.com/santifer/career-ops
44,469 stars
Taste skill that pushes agents away from generic output.
https://github.com/Leonxlnx/taste-skill
42,815 stars
Researches trends across Reddit, X, YouTube, HN, and the web.
https://github.com/mvanhorn/last30days-skill
The interesting part: 6 of the top 10 are single-skill repos.
Not huge libraries. One sharp capability.
Full post: https://generativeprogrammer.com/p/20-agent-skills-repos-and-marketplaces

We should seriously consider using the most popular skills as determined by marketplaces like skills.sh and others; there is likely a lot to learn at minimum from these beloved skills. Seriously consider each and every top skill using dedicated subagents, and determine if and how to incorporate them (whether wholecloth or in part) into harness-kit.

## What does "works" mean?

"""
Mitchell Hashimoto
@mitchellh
The problem with the "if it works who cares what the code looks like" mindset for agentic work is that it assumes the agent has a perfect understanding of "works." Realistically, things are underspecified, agents make bad assumptions, etc.

To be fair, agents are pretty good at unit test coverage. They're pretty bad at designing human experiences (API, CLI flags, etc.), especially cohesive ones for future roadmap plans they may not have visibility into (unless your backlog is perfect and vision fully laid out, which I doubt). They're bad at knowing where performance matters and what type (CPU vs memory tradeoffs). They're bad at where compatibility matters and where it doesn't (and tend to err on the side of preserving it without further guidance). Etc.

Unless you have this ALL specified, you can't possibly claim "it works" without taking a look and thinking about it.
"""

We sould work to mitigate this issue somehow.

## Loop Engineering

Read the loop-engineering.md file for the articles (link here, but you might not be able to read it that way)
https://x.com/0xCodez/status/2064374643729773029
https://x.com/samueljmcd/status/2066524627585634765

Should we incorporate some of this into harness-kit? Our misty-step/bitterblossom repo? Something else?

## HarnessX

"""

Akshay 🚀
@akshay_pachaar
HarnessX: a harness that compiles itself.

every harness improvement so far has come from a human editing code by hand.

Anthropic strips planning steps out of Claude Code when a stronger model ships. Manus rebuilt its agent five times in six months, removing complexity each round.

the craft runs on human judgment about what to change and when. HarnessX is what happens when a system makes those edits itself.

the trick is to treat the harness as a first-class object, the way we already treat model weights.

once it's a typed, editable artifact, it can be optimized from its own execution traces.

the framing they use is an operational mirror. evolving a harness maps cleanly onto reinforcement learning.

the harness is the state. an edit is the action. the trace plus a score is the feedback. a new version is the update.

once you see it that way, the failure modes come for free. reward hacking, catastrophic forgetting, under-exploration.

the same problems that break model training show up when a system edits its own scaffolding.

so edits never ship blind. each round, a loop reads the traces, plans a change, writes the edit, then critiques it.

a gate keeps the new version only if it beats the current one on tasks it hasn't seen.

what makes this safe is the structure underneath. the harness is built from typed components the system can swap without breaking the rest.

that is what compiles really means here. every candidate harness is type-checked before it runs.

here is the result that matters. the weakest model improved the most. the strongest barely moved.

an evolved harness closes the gaps a weak model cannot fix on its own. the weights never changed. the environment around them got smarter.

this is the natural next phase of harness engineering. we moved from weights, to context, to hand-built harnesses.

the harness was the last piece we still tuned by hand.

i wrote a deep dive on agent harness engineering a while back, covering the orchestration loop, tools, memory, context management, and everything that turns a stateless LLM into a capable agent. the article is below.

paper: HarnessX: A Composable, Adaptive, and Evolvable Agent Harness Foundry: http://arxiv.org/abs/2606.14249
"""

Read the anatomy-of-an-agent-harness.md file too. It's got a couple great articles.

What can we incorporate from this into harness-kit?

## Elon Musk's 5-Step Algorithm for Solving Any Problem

"""


See new posts
Conversation
Jaynit
@jaynitx
Elon Musk explains his 5-step algorithm for solving any problem:

"The most common mistake of smart engineers is to optimize a thing that should not exist."

"I have this very basic first principles algorithm that I run as a mantra."

Elon breaks it down:

Step 1: Question the requirements.

"Make the requirements less dumb. The requirements are always dumb to some degree, no matter how smart the person who gave you those requirements. You have to start there, because otherwise you could get the perfect answer to the wrong question."

Step 2: Try to delete it.

"Try to delete the part or the process step entirely. If you're not forced to put back at least 10% of what you delete, you're not deleting enough. Most people feel like they've succeeded if they haven't been forced to put things back in. But actually they haven't, they've been overly conservative and left things in that shouldn't be there."

Step 3: Optimize or simplify.

"The most common mistake of smart engineers is to optimize a thing that should not exist. So you don't optimize until after you've tried to delete."

Step 4: Speed it up.

"Any given thing can be done faster than you think. But you shouldn't speed things up until you've tried to delete it and optimize it otherwise, you're speeding up something that shouldn't exist."

Step 5: Automate.

"And then the fifth thing is to automate it."

Elon explains why the order matters:

"I've gone backwards so many times where I've automated something, sped it up, simplified it, and then deleted it. I got tired of doing that. So that's why I have this mantra."
"""

We should probably incorporate this into harness-kit somewhere / somehow.
