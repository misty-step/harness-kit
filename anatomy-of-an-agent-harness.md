The Anatomy of an Agent Harness
A deep dive into what Anthropic, OpenAI, Perplexity and LangChain are actually building. Covering the orchestration loop, tools, memory, context management, and everything else that transforms a stateless LLM into a capable agent.
You've built a chatbot. Maybe you've wired up a ReAct loop with a few tools. It works for demos. Then you try to build something production-grade, and the wheels come off: the model forgets what it did three steps ago, tool calls fail silently, and context windows fill up with garbage.
The problem isn't your model. It's everything around your model.
LangChain proved this when they changed only the infrastructure wrapping their LLM (same model, same weights) and jumped from outside the top 30 to rank 5 on TerminalBench 2.0. A separate research project hit a 76.4% pass rate by having an LLM optimize the infrastructure itself, surpassing hand-designed systems.
That infrastructure has a name now: the agent harness.
What Is the Agent Harness?
The term was formalized in early 2026, but the concept existed long before. The harness is the complete software infrastructure wrapping an LLM: orchestration loop, tools, memory, context management, state persistence, error handling, and guardrails. Anthropic's Claude Code documentation puts it simply: the SDK is "the agent harness that powers Claude Code." OpenAI's Codex team uses the same framing, explicitly equating the terms "agent" and "harness" to refer to the non-model infrastructure that makes the LLM useful.
I really liked the canonical formula, from LangChain's Vivek Trivedy: "If you're not the model, you're the harness."
Here's the distinction that trips people up. The "agent" is the emergent behavior: the goal-directed, tool-using, self-correcting entity the user interacts with. The harness is the machinery producing that behavior. When someone says "I built an agent," they mean they built a harness and pointed it at a model.
Beren Millidge made this analogy precise in his 2023 essay "Scaffolded LLMs as Natural Language Computers." A raw LLM is a CPU with no RAM, no disk, and no I/O. The context window serves as RAM (fast but limited). External databases function as disk storage (large but slow). Tool integrations act as device drivers. The harness is the operating system. As Millidge wrote: "We have reinvented the Von Neumann architecture" because it's a natural abstraction for any computing system.
Three Levels of Engineering
Three concentric levels of engineering surround the model:
Prompt engineering crafts the instructions the model receives.
Context engineering manages what the model sees and when.
Harness engineering encompasses both, plus the entire application infrastructure: tool orchestration, state persistence, error recovery, verification loops, safety enforcement, and lifecycle management.
The harness is not a wrapper around a prompt. It is the complete system that makes autonomous agent behavior possible.
The 12 Components of a Production Harness
Synthesizing across Anthropic, OpenAI, LangChain, and the broader practitioner community, a production agent harness has twelve distinct components. Let's walk through each one.
1. The Orchestration Loop
This is the heartbeat. It implements the Thought-Action-Observation (TAO) cycle, also called the ReAct loop. The loop runs: assemble prompt, call LLM, parse output, execute any tool calls, feed results back, repeat until done.
Mechanically, it's often just a while loop. The complexity lives in everything the loop manages, not the loop itself. Anthropic describes their runtime as a "dumb loop" where all intelligence lives in the model. The harness just manages turns.
2. Tools
Tools are the agent's hands. They're defined as schemas (name, description, parameter types) injected into the LLM's context so the model knows what's available. The tool layer handles registration, schema validation, argument extraction, sandboxed execution, result capture, and formatting results back into LLM-readable observations.
Claude Code provides tools across six categories: file operations, search, execution, web access, code intelligence, and subagent spawning. OpenAI's Agents SDK supports function tools (via @function_tool), hosted tools (WebSearch, CodeInterpreter, FileSearch), and MCP server tools.
3. Memory
Memory operates at multiple timescales. Short-term memory is conversation history within a single session. Long-term memory persists across sessions: Anthropic uses CLAUDE.md project files and auto-generated MEMORY.md files; LangGraph uses namespace-organized JSON Stores; OpenAI supports Sessions backed by SQLite or Redis.
Claude Code implements a three-tier hierarchy: a lightweight index (~150 characters per entry, always loaded), detailed topic files pulled in on demand, and raw transcripts accessed via search only. A critical design principle: the agent treats its own memory as a "hint" and verifies against actual state before acting.
4. Context Management
This is where many agents fail silently. The core problem is context rot: model performance degrades 30%+ when key content falls in mid-window positions (Chroma research, corroborated by Stanford's "Lost in the Middle" finding). Even million-token windows suffer from instruction-following degradation as context grows.
Production strategies include:
Compaction: summarizing conversation history when approaching limits (Claude Code preserves architectural decisions and unresolved bugs while discarding redundant tool outputs)
Observation masking: JetBrains' Junie hides old tool outputs while keeping tool calls visible
Just-in-time retrieval: maintaining lightweight identifiers and loading data dynamically (Claude Code uses grep, glob, head, tail rather than loading full files)
Sub-agent delegation: each subagent explores extensively but returns only 1,000 to 2,000 token condensed summaries
Anthropic's context engineering guide states the goal: find the smallest possible set of high-signal tokens that maximize likelihood of the desired outcome.
5. Prompt Construction
This assembles what the model actually sees at each step. It's hierarchical: system prompt, tool definitions, memory files, conversation history, and the current user message.
OpenAI's Codex uses a strict priority stack: server-controlled system message (highest priority), tool definitions, developer instructions, user instructions (cascading AGENTS.md files, 32 KiB limit), then conversation history.
6. Output Parsing
Modern harnesses rely on native tool calling, where the model returns structured tool_calls objects rather than free-text that must be parsed. The harness checks: are there tool calls? Execute them and loop. No tool calls? That's the final answer.
For structured outputs, both OpenAI and LangChain support schema-constrained responses via Pydantic models. Legacy approaches like RetryWithErrorOutputParser (which feeds the original prompt, the failed completion, and the parsing error back to the model) remain available for edge cases.
7. State Management
LangGraph models state as typed dictionaries flowing through graph nodes, with reducers merging updates. Checkpointing happens at super-step boundaries, enabling resume after interruptions and time-travel debugging. OpenAI offers four mutually exclusive strategies: application memory, SDK sessions, server-side Conversations API, or lightweight previous_response_id chaining. Claude Code takes a different approach: git commits as checkpoints and progress files as structured scratchpads.
8. Error Handling
Here's why this matters: a 10-step process with 99% per-step success still has only ~90.4% end-to-end success. Errors compound fast.
LangGraph distinguishes four error types: transient (retry with backoff), LLM-recoverable (return error as ToolMessage so the model can adjust), user-fixable (interrupt for human input), and unexpected (bubble up for debugging). Anthropic catches failures within tool handlers and returns them as error results to keep the loop running. Stripe's production harness caps retry attempts at two.
9. Guardrails and Safety
OpenAI's SDK implements three levels: input guardrails (run on first agent), output guardrails (run on final output), and tool guardrails (run on every tool invocation). A "tripwire" mechanism halts the agent immediately when triggered.
Anthropic separates permission enforcement from model reasoning architecturally. The model decides what to attempt; the tool system decides what's allowed. Claude Code gates ~40 discrete tool capabilities independently, with three stages: trust establishment at project load, permission check before each tool call, and explicit user confirmation for high-risk operations.
10. Verification Loops
This is what separates toy demos from production agents. Anthropic recommends three approaches: rules-based feedback (tests, linters, type checkers), visual feedback (screenshots via Playwright for UI tasks), and LLM-as-judge (a separate subagent evaluates output).
Boris Cherny, creator of Claude Code, noted that giving the model a way to verify its work improves quality by 2 to 3x.
11. Subagent Orchestration
Claude Code supports three execution models: Fork (byte-identical copy of parent context), Teammate (separate terminal pane with file-based mailbox communication), and Worktree (own git worktree, isolated branch per agent). OpenAI's SDK supports agents-as-tools (specialist handles bounded subtask) and handoffs (specialist takes full control). LangGraph implements subagents as nested state graphs.
The Loop in Motion: A Step-by-Step Walkthrough
Now that you know the components, let's trace how they work together in a single cycle.
Step 1 (Prompt Assembly): The harness constructs the full input: system prompt + tool schemas + memory files + conversation history + current user message. Important context is positioned at the beginning and end of the prompt (the "Lost in the Middle" finding).
Step 2 (LLM Inference): The assembled prompt goes to the model API. The model generates output tokens: text, tool call requests, or both.
Step 3 (Output Classification): If the model produced text with no tool calls, the loop ends. If it requested tool calls, proceed to execution. If a handoff was requested, update the current agent and restart.
Step 4 (Tool Execution): For each tool call, the harness validates arguments, checks permissions, executes in a sandboxed environment, and captures results. Read-only operations can run concurrently; mutating operations run serially.
Step 5 (Result Packaging): Tool results are formatted as LLM-readable messages. Errors are caught and returned as error results so the model can self-correct.
Step 6 (Context Update): Results are appended to conversation history. If approaching the context window limit, the harness triggers compaction.
Step 7 (Loop): Return to Step 1. Repeat until termination.
Termination conditions are layered: the model produces a response with no tool calls, maximum turn limit is exceeded, token budget is exhausted, a guardrail tripwire fires, the user interrupts, or a safety refusal is returned. A simple question might take 1 to 2 turns. A complex refactoring task can chain dozens of tool calls across many turns.
For long-running tasks spanning multiple context windows, Anthropic developed a two-phase "Ralph Loop" pattern: an Initializer Agent sets up the environment (init script, progress file, feature list, initial git commit), then a Coding Agent in every subsequent session reads git logs and progress files to orient itself, picks the highest-priority incomplete feature, works on it, commits, and writes summaries. The filesystem provides continuity across context windows.
How Real Frameworks Implement the Pattern
Anthropic's Claude Agent SDK exposes the harness through a single query() function that creates the agentic loop and returns an async iterator streaming messages. The runtime is a "dumb loop." All intelligence lives in the model. Claude Code uses a Gather-Act-Verify cycle: gather context (search files, read code), take action (edit files, run commands), verify results (run tests, check output), repeat.
OpenAI's Agents SDK implements the harness through the Runner class with three modes: async, sync, and streamed. The SDK is "code-first": workflow logic is expressed in native Python rather than graph DSLs. The Codex harness extends this with a three-layer architecture: Codex Core (agent code + runtime), App Server (bidirectional JSON-RPC API), and client surfaces (CLI, VS Code, web app). All surfaces share the same harness, which is why "Codex models feel better on Codex surfaces than a generic chat window."
LangGraph models the harness as an explicit state graph. Two nodes (llm_call and tool_node) connected by a conditional edge: if tool calls present, route to tool_node; if absent, route to END. LangGraph evolved from LangChain's AgentExecutor, which was deprecated in v0.2 because it was hard to extend and lacked multi-agent support. LangChain's Deep Agents explicitly use the term "agent harness": built-in tools, planning (write_todos tool), file systems for context management, subagent spawning, and persistent memory.
CrewAI implements a role-based multi-agent architecture: Agent (the harness around the LLM, defined by role, goal, backstory, and tools), Task (the unit of work), and Crew (the collection of agents). CrewAI's Flows layer adds a "deterministic backbone with intelligence where it matters," managing routing and validation while Crews handle autonomous collaboration.
AutoGen (evolving into Microsoft Agent Framework) pioneered conversation-driven orchestration. Its three-layer architecture (Core, AgentChat, Extensions) supports five orchestration patterns: sequential, concurrent (fan-out/fan-in), group chat, handoff, and magentic (a manager agent maintains a dynamic task ledger coordinating specialists).
The Scaffolding Metaphor
The scaffolding metaphor isn't decorative. It's precise. Construction scaffolding is temporary infrastructure that enables workers to build a structure they couldn't reach otherwise. It doesn't do the construction. But without it, workers can't reach the upper floors.
The key insight: scaffolding is removed when the building is complete. As models improve, harness complexity should decrease. Manus was rebuilt five times in six months, each rewrite removing complexity. Complex tool definitions became general shell execution. "Management agents" became simple structured handoffs.
This points to the co-evolution principle: models are now post-trained with specific harnesses in the loop. Claude Code's model learned to use the specific harness it was trained with. Changing tool implementations can degrade performance because of this tight coupling.
The "future-proofing test" for harness design: if performance scales up with more powerful models without adding harness complexity, the design is sound.
Seven Decisions That Define Every Harness
Every harness architect faces seven choices:
Single-agent vs. multi-agent. Both Anthropic and OpenAI say: maximize a single agent first. Multi-agent systems add overhead (extra LLM calls for routing, context loss during handoffs). Split only when tool overload exceeds ~10 overlapping tools or clearly separate task domains exist.
ReAct vs. plan-and-execute. ReAct interleaves reasoning and action at every step (flexible but higher per-step cost). Plan-and-execute separates planning from execution. LLMCompiler reports a 3.6x speedup over sequential ReAct.
Context window management strategy. Five production approaches: time-based clearing, conversation summarization, observation masking, structured note-taking, and sub-agent delegation. ACON research showed 26 to 54% token reduction while preserving 95%+ accuracy by prioritizing reasoning traces over raw tool outputs.
Verification loop design. Computational verification (tests, linters) provides deterministic ground truth. Inferential verification (LLM-as-judge) catches semantic issues but adds latency. Martin Fowler's Thoughtworks team frames this as guides (feedforward, steer before action) versus sensors (feedback, observe after action).
Permission and safety architecture. Permissive (fast but risky, auto-approve most actions) versus restrictive (safe but slow, require approval for each action). The choice depends on deployment context.
Tool scoping strategy. More tools often means worse performance. Vercel removed 80% of tools from v0 and got better results. Claude Code achieves 95% context reduction via lazy loading. The principle: expose the minimum tool set needed for the current step.
Harness thickness. How much logic lives in the harness versus the model. Anthropic bets on thin harnesses and model improvement. Graph-based frameworks bet on explicit control. Anthropic regularly deletes planning steps from Claude Code's harness as new model versions internalize that capability.
The Harness Is the Product
Two products using identical models can have wildly different performance based solely on harness design. The TerminalBench evidence is clear: changing only the harness moved agents by 20+ ranking positions.
The harness is not a solved problem or a commodity layer. It's where the hard engineering lives: managing context as a scarce resource, designing verification loops that catch failures before they compound, building memory systems that provide continuity without hallucination, and making architectural bets about how much scaffolding to build versus how much to leave to the model.
The field is moving toward thinner harnesses as models improve. But the harness itself isn't going away. Even the most capable model needs something to manage its context window, execute its tool calls, persist its state, and verify its work.
The next time your agent fails, don't blame the model. Look at the harness.
That's a wrap!
If you enjoyed reading this:
Find me →@akshay_pachaar ✔️
Every day, I share tutorials and insights on AI, Machine Learning, and vibe coding best practices.

---

Agent harness engineering with Claude: 14-step roadmap from one agent to a self-improving system.
Everyone’s talking about loops. Almost no one is talking about what the loop runs on. 9 out of 10 builders run Claude Code on the default harness - no rules, no subagents, no hooks, no memory.
Then they wonder why their loop produces slop. The truth is simple: a loop is only as good as the harness underneath it. This is the 14-step roadmap to the harness - from one agent to a system that improves itself.
Follow my Substack to get fresh AI alpha: movez.substack.com
Loop engineering - building a system that prompts your agent on a schedule - got all the attention this month. But Addy Osmani, who wrote the long-form piece on loops, was careful to point at what sits below it:
“Loop engineering sits one floor above the harness. The harness is the environment one single agent runs inside. The loop is the harness, but it runs on a timer, spawns helpers, and feeds itself.”
Harness engineering is designing that environment: the model, the tools, the permissions, the context, the memory.
It’s the unglamorous layer - and it’s the one that decides whether everything above it works. A great loop on a bad harness is a fast way to produce garbage at scale.
14 steps. 3 tiers. The foundation everything else stands on.

Part 1 · what Harness is
01. A harness is the environment one agent runs inside.
Strip away the jargon and a harness is four things: the model doing the thinking, the tools it can reach, the permissions on those tools, and the context it reads at the start of every run.
That’s the whole surface. Everything else - subagents, hooks, memory - is a way of shaping one of those four.
The reason harness matters more than people think: the agent is a while True loop that picks a tool, runs it, looks at the result, and decides the next move.
The harness defines what tools exist, what the agent is allowed to do, and what it knows when it starts. Same model, different harness, completely different agent.

02. The whole harness lives in one folder. .claude/
Everything that shapes your agent sits in a single directory at your project root. Learn this layout and you can read anyone’s harness at a glance:
python
.claude/
├─ CLAUDE.md          # standing facts — read every session
├─ settings.json      # permissions, model, hooks
├─ .mcp.json          # external tool connections
├─ rules/             # path-scoped behaviors
│  ├─ tests.md
│  └─ python-types.md
├─ agents/            # subagent definitions (~30 lines each)
│  ├─ reviewer.md
│  └─ eval-runner.md
├─ skills/            # reusable workflows
│  └─ pr-checklist/
│     └─ SKILL.md
└─ agent-memory/      # what survives between runs
   └─ STATE.md
One rule that separates a clean harness from a mess: keep it small enough that you can explain why every file exists. If you can’t say what a rule, hook, or subagent is for, delete it.

03. Harness vs loop vs system. Three floors, don’t mix them.
Most “my agent setup is a mess” problems come from confusing the three floors. Keep them straight:
The harness is the runtime one agent lives in. Static configuration: model, tools, permissions, context. This issue.
The loop prompts the agent on a timer, spawns helpers, feeds itself. It runs on top of the harness.
The self-improving system is a loop plus memory that compounds - every run leaves the next run sharper.
The practical version: put standing facts in context, enforcement in hooks, procedures in skills, and isolation in subagents.
Mixing these up - enforcement in CLAUDE.md, procedures bloating context - is the root cause of inconsistent, expensive agents.

04. The default harness. What you get out of the box.
Install Claude Code, open a folder, and you already have a harness - just an empty one. The default gives you a capable model, the built-in tools (read, write, bash, search), and approval prompts on everything risky. No project context, no custom subagents, no memory.
For a one-off task, the default is fine. For anything you do more than once, the default leaves the agent re-deriving your project from scratch every session, asking permission for safe operations, and forgetting everything when you close the terminal.
The next ten steps are about closing that gap.

05. CLAUDE.md: standing facts, kept short.
CLAUDE.md is read at the start of every session. It’s the agent’s standing knowledge of your project - conventions, architecture, the “we don’t do it this way because of that incident.”
The single most common mistake: letting it grow into a giant procedures document that bloats every session.
The rule from practitioners running this daily: keep the main memory file under ~500 tokens. Standing facts go here.
Multi-step procedures go in skills (step 8). Path-specific behaviors go in rules/ files scoped to where they apply. If a section of CLAUDE.md has become a procedure rather than a fact, it belongs somewhere else.
Read your CLAUDE.md out loud. Every line should be a fact the agent needs in every session (“we use pnpm, not npm”). If a line is a procedure (“to add a feature, first…”), move it to a skill.
If it’s a rule for one folder, move it to rules/.

06. settings.json: permissions and model, set once.
The default harness asks before every risky action. That’s right when you’re watching and wrong when you’re not. settings.json is where you pre-approve the safe stuff, deny the dangerous stuff, and pick which model runs.
python
{
  "model": "claude-sonnet-4-6",
  "permissions": {
    "autoApprove": [
      "Read(*)", "Grep(*)",
      "Bash(npm test)", "Bash(git status)"
    ],
    "deny": [
      "Bash(rm -rf*)", "Bash(git push*)",
      "Edit(.env*)", "Edit(secrets/*)"
    ]
  }
}
The test for what to auto-approve: if this goes wrong, how hard is it to undo? Cheap to undo → auto-approve.
Expensive to undo (force-push, deleting files, touching secrets) → always deny or prompt. The middle ground is fine to auto-approve if you log it.

07. Subagents: isolated context for the dirty work.
A subagent is an independent Claude session launched from the main one - its own context window, its own tool list. The point isn’t parallelism for its own sake. It’s keeping noise out of the main context.
A research task that reads 40 files, a review pass that needs a fresh perspective, an eval run that produces a wall of logs - those belong in a subagent so they don’t pollute the main thread.
The most valuable subagent in any harness is the one that checks work the main agent did. A model reviewing its own output is too easy on itself;
A separate reviewer with a fresh context window catches what the writer talked itself into. This is the writer-vs-checker split that makes every loop above the harness trustworthy.

08. Skills: procedures the agent reuses.
A Skill is a SKILL.md file the agent runs - either when you call it with /skill-name or automatically when the task matches its description.
Unlike a subagent, it runs in the same context window. It’s just reusable instructions that become part of the session.
The trigger to create one: you notice yourself pasting the same instructions into every new conversation. That’s a skill waiting to happen. A PR checklist, an eval procedure, a release process - written once, invoked forever.
And because skills are the reusable unit, they’re what makes the harness improve over time: each time the procedure fails in a new way, you add the lesson to the skill, and the next run inherits it.

09. Hooks: deterministic rules the model can’t hallucinate.
Everything so far depends on the model understanding your instructions. Hooks don’t.
A hook is a shell command that fires at a fixed point in the agent lifecycle - before a tool runs, after a file changes, when the session ends- and its exit code can block the action. Hooks are enforcement, CLAUDE.md is suggestion.
Two hooks earn their place in almost every harness:
A PreToolUse gate that blocks dangerous commands deterministically — rm -rf, reading .env, pushing to main. Exit code 2 stops the call before it happens. The model can’t talk its way past it.
A PostToolUse formatter that runs your linter or formatter after every edit. The agent never ships unformatted code because the harness formats it automatically.
python
"hooks": {
  "PreToolUse": [{
    "matcher": "Bash",
    "command": "./.claude/hooks/block-dangerous.sh"
    // exit 2 = block the call before it runs
  }],
  "PostToolUse": [{
    "matcher": "Edit|Write",
    "command": "prettier --write \"$CLAUDE_FILE_PATH\""
  }]
}
Use hooks for anything that must happen or must never happen - safety, formatting, audit logging.
Don’t use them for judgment calls; that’s what the model is for. A good harness has one or two sharp hooks, not twenty.

Part 3 · make It Compound
10. Add a loop. Now the harness runs on a timer.
A configured harness still waits for you to type. A loop makes it run on its own. The simplest version is /loop in Claude Code - a recurring prompt on a cadence.
Pair it with /goal and the loop keeps going until an objective condition is true, checked by an independent grader rather than the agent grading itself.
python
> /loop 30m /goal All tests pass and lint is clean.
  Triage new failures, draft fixes in claude/ branches.

▲ Claude uses the harness you built:
  - rules/ for conventions
  - reviewer subagent to check each fix
  - PreToolUse hook blocks pushes to main
✓ Looping. Independent grader decides “done.”
Notice what just happened: the loop didn’t add intelligence. It re-used everything in the harness - the rules, the reviewer subagent, the safety hook. A good harness makes a loop trivial. That’s the whole point of building the foundation first.

11. Add dynamic workflows. The harness writes its own orchestration.
For tasks too complex for a single loop - massively parallel, highly structured, adversaria- Claude can write its own JavaScript harness on the fly.
That’s a dynamic workflow: agent() to spawn, parallel() to fan out, pipeline() to stream. It composes the subagents your harness defines into patterns like fan-out-and-synthesize or adversarial verification.
The connection to harness engineering: a dynamic workflow is only as good as the subagents and skills it can call.
If your harness has a sharp reviewer subagent and a well-written eval skill, the workflow has good pieces to orchestrate. If the harness is empty, the workflow has nothing to work with.
The workflow is the conductor, your harness is the orchestra.

12. Add memory. What the agent forgets, the harness remembers.
This is the step that turns a configured harness into a system that actually improves. The agent forgets everything between runs. The harness doesn’t have to.
A state file - a markdown file in agent-memory/, or a Linear board - records what was tried, what worked, what failed, what rules survived.
The pattern that makes memory compound, drawn from how the strongest agents use it:
Write before walking away. Every run ends by updating the state file - lessons learned, verified facts, what’s next.
Read at the start. Every run begins by reading the state file and relevant skills, so it resumes instead of restarting.
Distill into skills. When a lesson is general (“Windows runners need bash, not PowerShell”), it graduates from the state file into a skill, where it applies to every future project.
python
# Project memory

## Verified facts # stop guessing about these
- prc is in dollars, not cents (checked via SELECT MIN/MAX)
- auth middleware order: rate_limit -> jwt -> rbac

## Lessons learned # distill the general ones into skills
- Windows CI runners fail TLS 1.2 in PowerShell — use bash
- Migrations on tables >1M rows must batch in 10k chunks

## Last session # resume, don’t restart
2026-06-11 · 3 fixes merged, 2 escalated. Next: verify rate-limit fix.

13. Close the loop. Output → lesson → skill → better output.
Here’s where the three floors lock together into something that improves itself. Each run produces output. The reviewer subagent (step 7) checks it.
The result - what passed, what failed, what was learned - gets written to memory (step 12). The general lessons get distilled into skills (step 8).
The next run inherits sharper skills and richer memory.
That’s the whole self-improving loop, and notice it’s built entirely from harness parts:
Subagent grades the work - objective check, fresh context.
Memory records the verdict - survives between runs.
Skills a runs it again - now with everything the last run learned.
The loop runs it again - now with everything the last run learned.
The model never changed. The harness around it got sharper. That’s what “self-improving” honestly means - not a model that learns, but a harness that accumulates.

14. Ship the harness. Package it. Share it. Reuse it.
A harness that works on one project is an asset.
Bundle the skills, subagents, and rules into a plugin and your whole team installs the same setup in one step - same conventions, same safety hooks, same reviewer.
The harness stops being your personal setup and becomes shared infrastructure.
The order to build, one last time, because order is the lesson: get one manual run reliable on a clean harness.
Add the context and permissions. Add a reviewer subagent. Add memory. Then  and only then wrap it in a loop. A loop on a good harness compounds. A loop on a bad harness just bleeds faster.

§ The harness mistakes that make every loop worse
Running on the default. No context, no rules, no memory - the agent re-derives your project every session.
A bloated CLAUDE.md. Procedures stuffed into standing context, bloating every run. Move them to skills.
Enforcement in CLAUDE.md instead of hooks. The model can ignore a suggestion. It can’t ignore a hook that exits 2.
One agent writing and grading its own work. Add a reviewer subagent with a fresh context window.
No memory. Every run restarts from zero. The state file is what makes tomorrow resume.
Wrapping a loop around a bad harness. The loop just produces slop faster. Build the foundation first.
Twenty hooks. One or two sharp ones beat a pile nobody understands.
Shipping a harness without scanning it. Leaked secrets and over-broad permissions spread to everyone who installs it.

Conclusion:
The loop gets the glory. The harness does the work.
Loop engineering is the exciting part - the agent prompting itself, running while you sleep. But a loop is just a harness on a timer.
Everything that decides whether the output is good or garbage lives one floor down, in the model you picked, the tools you allowed, the context you wrote, the reviewer you added, the memory you kept.
Build that floor well and everything above it compounds: the loop re-uses your subagents, the workflow orchestrates your skills, the memory makes each run sharper than the last.
Self-improvement was never a property of the model. It’s a property of the harness you build around it.
Pick one thing you’re not doing - probably a reviewer subagent, a safety hook, or a state file — and add it today. Keep the harness small enough to explain. Then put a loop on top, and watch the foundation do the work.

