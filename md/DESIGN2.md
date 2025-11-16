# ttyclaude - Design Document v2
## VPS & Remote Self-Hosted LLMs

**Author's Note**: This is a companion to DESIGN.md, focusing specifically on the VPS/remote self-hosted use case that's central to many hacker workflows. Read this if you're thinking "I want my own models, but my laptop is weak and I have a VPS with decent specs sitting idle."

## The VPS Use Case

You've got a VPS. Maybe it's $10/mo Digital Ocean, maybe it's a chunky Hetzner box, maybe it's that underutilized slice at work. It's got 16GB RAM, decent CPU, and sits there 24/7 connected to good bandwidth. Your laptop? It's a 8GB ultralight that's perfect for travel but chokes on LLM inference.

**The solution**: Run your own models on the VPS. Access them from anywhere. Own your data. Pay once (VPS fee), use unlimited.

### Why This Matters

**Privacy**: Your conversations never hit Anthropic, OpenAI, or anyone's servers. It's your VPS, your model, your data.

**Cost**: API calls add up. $20/mo for Claude if you use it heavily. Or... $10/mo VPS running phi-4 unlimited.

**Offline-ish**: Your VPS is always on. SSH into it from anywhere, ttyclaude connects to your lmcli instance, you're chatting.

**Control**: Want to fine-tune on domain-specific data? Go ahead, it's your model. Want logs? Keep them. Want to experiment with prompts? No rate limits.

**Latency**: VPS in same datacenter as your home connection = faster than cross-country API calls.

## lmstudio & lmcli

**lmstudio** is a desktop app for running local models. Great UI, handles GGUF files, makes local LLMs accessible. But the real magic for terminal users is **lmcli** - the CLI/server component.

**lmcli** gives you an OpenAI-compatible API endpoint. This is huge: it means any tool that works with OpenAI's API can work with your self-hosted models. Just point it at `http://your-vps:8080` instead of `api.openai.com`.

For ttyclaude, this means lmcli is just another provider:

```
[providers.vps-lmcli]
type = "openai-compatible"  # lmcli speaks OpenAI API
endpoint = "https://my-vps.example.com:8080"
api_key = "optional-your-token-here"
models = ["phi-4-mini-reasoning"]
default_model = "phi-4-mini-reasoning"
```

## Model Selection: Size Matters

When you're running on a VPS, you're not running 70B parameter monsters. You're running models that fit in RAM with headroom for the OS and other stuff.

**microsoft/phi-4-mini-reasoning** at 2.49GB (Q4_K_M quantization) is the sweet spot:
- Fits comfortably in 8GB RAM VPS (leaves room for OS)
- Runs on 16GB VPS with plenty of breathing room
- Still shockingly capable for coding, reasoning, quick questions
- Fast inference on decent CPU
- Perfect for the "quick check" use case

Other candidates for VPS deployment:
- **mistral-7b-instruct** (~4GB): Solid general purpose
- **codellama-7b** (~4GB): Code-focused if that's your jam  
- **tinyllama-1.1b** (~0.7GB): Blazing fast, good for simple tasks
- **deepseek-coder-1.3b** (~0.8GB): Surprisingly good at code despite size

The pattern: 1-7B parameters, quantized, < 5GB disk space. These are "good enough" for 80% of use cases while being VPS-friendly.

## The Multi-Provider Workflow (Extended)

Original vision had three buffers:
1. anthropic/opus - deep thinking
2. local/mistral - quick checks
3. openai/gpt4 - second opinions

Now add the VPS dimension:

```
Buffer 1: anthropic/opus
  Use case: Really hard problems, architecture decisions
  Cost: API fees, worth it for critical thinking
  
Buffer 2: vps-lmcli/phi-4
  Use case: Code review, quick questions, iterations
  Cost: VPS fee (already paying anyway)
  Speed: Fast, your VPS has low latency
  Privacy: 100%, it's your server
  
Buffer 3: local/mistral
  Use case: Offline work, train WiFi, truly private
  Cost: Free, uses laptop CPU
  
Buffer 4: openai/gpt4  
  Use case: Second opinion, team compatibility
  Cost: API fees
```

Alt-1/2/3/4 to switch contexts. Use the right tool for the job. The VPS model becomes your workhorse for everyday stuff.

## Deployment Scenarios

### Scenario 1: Personal VPS
You've got a $10/mo Digital Ocean droplet. Clone ttyclaude, clone lmcli, download phi-4-mini, run it. Point your local ttyclaude at your VPS. Done.

**Pros**: Total control, always accessible, good for personal projects  
**Cons**: You're the sysadmin (but you knew that)

### Scenario 2: Work VPS
Your company has internal VPS infrastructure. Deploy lmcli on an internal box. Now your whole team can use the same self-hosted model.

**Pros**: Company data never leaves company network, shared resource  
**Cons**: Need buy-in, someone has to maintain it

### Scenario 3: Home Server + SSH Tunnel
Run lmcli on your home server (old desktop, NAS, whatever). SSH tunnel to access it remotely.

**Pros**: Use hardware you already own, truly private  
**Cons**: Depends on home internet uptime, need SSH access

### Scenario 4: GPU VPS (If Budget Allows)
Rent a cheap GPU instance ($30-50/mo). Now you can run 13B+ models fast.

**Pros**: Better models, still self-hosted, faster inference  
**Cons**: More expensive (but still cheaper than heavy API usage)

## Provider Types: The Full Picture

ttyclaude needs to handle these provider patterns:

**1. Commercial APIs** (anthropic, openai)
- HTTPS to their endpoints
- API key auth
- Pay per token
- High quality, no hardware needed

**2. Local Inference** (llama.cpp on localhost)
- HTTP to localhost:8080
- No auth typically
- Free after model download
- Uses your laptop resources

**3. Self-Hosted Remote** (lmcli on VPS, ollama on server)
- HTTPS to your server (or SSH tunnel)
- Optional auth (you control it)
- Fixed cost (VPS fee)
- Your hardware, your control

**4. OpenAI-Compatible** (lmcli, vllm, text-generation-webui)
- Any server speaking OpenAI API format
- Could be internal company deployment
- Could be someone else's self-hosted instance you have access to
- Same interface as OpenAI, different backend

The beauty: ttyclaude doesn't care. They all implement the same interface. As long as messages go in and tokens stream out, it's a provider.

## Configuration Philosophy

The config should make it trivial to add providers. Don't overthink it:

```toml
[providers.my-vps]
type = "openai-compatible"
endpoint = "https://vps.example.com:8080"
# api_key = "optional"
models = ["phi-4-mini-reasoning", "mistral-7b"]
default_model = "phi-4-mini-reasoning"

[providers.home-server]
type = "openai-compatible"  
endpoint = "http://localhost:8080"  # SSH tunnel to home
models = ["codellama-7b"]
default_model = "codellama-7b"
```

That's it. No special cases, no complex setup. If it speaks OpenAI API (and most self-hosted solutions do), it works.

## Security Considerations for VPS Deployment

Running your own LLM server means thinking about security:

**HTTPS**: Use it. Let's Encrypt is free. Don't send prompts over plain HTTP.

**Authentication**: lmcli and most servers support API keys. Use them. Don't expose unauthenticated LLM endpoints to the internet.

**Firewall**: Only open the ports you need. If it's just for you, SSH tunnel instead of exposing the port.

**Monitoring**: Watch your VPS resources. An exposed LLM endpoint could get hammered by bots or abused. Set up basic alerting.

**Rate Limiting**: Most self-hosted tools support this. Use it to prevent abuse.

**Logs**: You control the logs. Decide what you keep, what you don't. Set up rotation so they don't fill your disk.

## Why Small Models Don't Suck

There's a bias that bigger = better. For some tasks, sure. But small models have been improving rapidly:

**phi-4-mini** (3.8B params) can:
- Write decent code
- Explain concepts clearly
- Debug simple issues
- Format data
- Answer factual questions

What it can't do as well:
- Deep philosophical reasoning
- Really complex multi-step problems
- Nuanced creative writing
- Remembering huge contexts

**The trick**: Use the right model for the task. phi-4 on your VPS handles 80% of daily work. Fall back to Claude Opus for the 20% that needs serious horsepower.

**Speed matters too**: Small model on decent VPS = sub-second first token. That snappy response time makes it feel more like a conversation, less like waiting for an API.

## Economics Breakdown

Let's do the math:

**Heavy API Usage**:
- Claude Opus: ~$15/million input tokens, ~$75/million output
- Moderate daily use: ~500k tokens/mo
- Cost: $20-40/mo depending on input/output ratio

**VPS + Small Model**:
- VPS: $10-20/mo (16GB RAM, decent CPU)
- Model: Free (phi-4, mistral, etc. are open weights)
- Usage: Unlimited
- Cost: $10-20/mo fixed

**GPU VPS + Bigger Model**:
- GPU VPS: $30-50/mo (A4000/similar)
- Model: Free (13B-30B open models)
- Usage: Unlimited, faster
- Cost: $30-50/mo fixed

**The crossover point**: If you're using APIs lightly, they're cheaper. If you're a heavy user (lots of code review, constant questions, experimentation), self-hosted VPS pays for itself fast.

Plus: No usage anxiety. With APIs you're always thinking "is this worth the tokens?" With self-hosted, ask away.

## Integration with ttyclaude Workflow

The screen/tmux workflow becomes even more powerful:

```
screen -S work

Window 0: ttyclaude
  Alt-1: vps-lmcli/phi-4        # Primary buffer, everyday use
  Alt-2: anthropic/opus         # Heavy lifting when needed
  Alt-3: local/mistral          # Offline fallback
  Alt-4: openai/gpt4            # Occasional second opinion

Window 1: claude                # Claude Code for tactical execution
Window 2: ssh to VPS            # Monitor lmcli, check resources
Window 3: vim                   # Actual work

Ctrl-a 0: Strategic thinking with multiple LLMs
Ctrl-a 1: Tactical execution
Ctrl-a 2: Server management
Ctrl-a 3: Coding
```

Everything in the terminal, everything one key combo away, everything under your control.

## Provider Discovery & Auto-Config

Future idea: ttyclaude could probe for common self-hosted endpoints:

- Check localhost:8080 (common llama.cpp port)
- Check localhost:5000 (common lmstudio port)
- Check localhost:1234 (another common port)
- SSH into known hosts and probe (if configured)

If found, auto-add to provider list with sane defaults. Make it zero-config for common setups.

This is where the IRC metaphor really shines: you'd do the same thing with IRC - client checks for bouncer on localhost, checks configured servers, shows you what's available.

## The Fallback Hierarchy

When your primary provider fails (network issue, server restart, rate limit), ttyclaude should offer smart fallbacks:

```
Trying: vps-lmcli/phi-4... [failed: connection refused]
Falling back to: local/mistral... [success]

Buffer 1 now using: local/mistral (temporary)
Original provider: vps-lmcli/phi-4 (will retry)
```

Or prompt the user:
```
vps-lmcli/phi-4 unavailable. Switch to:
1. local/mistral (free, slower)
2. anthropic/sonnet (fast, uses API credits)
3. Retry vps-lmcli
4. Cancel
```

Resilience through diversity. Having multiple providers isn't just about features, it's about reliability.

## Self-Hosted Model Ecosystem

The ecosystem is maturing fast:

**llama.cpp**: The OG, supports everything, fast, C++  
**lmstudio/lmcli**: Great UX, makes local models accessible  
**ollama**: Mac-focused but cross-platform, excellent for desktop use  
**vllm**: Production-grade, fast, Python, good for serious deployments  
**text-generation-webui**: Feature-rich, web UI + API, popular for experimentation  

Most speak OpenAI-compatible APIs now. This is the standard interface. ttyclaude supporting "openai-compatible" as a provider type covers all of these.

## Latency Characteristics

Understanding response times helps choose the right provider:

**Commercial APIs** (Anthropic, OpenAI):
- First token: 500-2000ms (network + queue)
- Streaming: Fast once started
- Variability: Depends on load, distance

**Local Laptop**:
- First token: 200-1000ms (pure compute)
- Streaming: Varies with hardware
- Consistency: Depends on laptop load

**VPS Self-Hosted**:
- First token: 200-800ms (network + light compute)
- Streaming: Consistent, good VPS = good perf
- Predictability: High, you control resources

**VPS on Good Connection**:
- Often faster than commercial APIs
- Lower variability
- No queueing behind other users

This is why VPS small models feel so snappy for quick questions. Sub-second first token, consistent streaming, no surprises.

## The Vision: True Multi-Provider Fluidity

Imagine your actual workflow:

**Morning standup prep**: Quick questions to vps-lmcli/phi-4. "Summarize yesterday's commits." Fast, free, done.

**Architecture decision**: Switch to anthropic/opus. This is hard, worth the API cost. Deep discussion about tradeoffs.

**Code review iteration**: Back to vps-lmcli/phi-4. Multiple rounds of "does this look right?" No cost anxiety, iterate freely.

**Offline on train**: Switch to local/mistral. No network needed, keep working.

**Team discussion needs GPT-4 output**: Switch to openai/gpt4. Someone on team wants to see GPT's take.

All in one tool. All in your terminal. All one Alt-N away. The right model for the right context, chosen in real-time.

## Why This Matters

This isn't about replacing commercial APIs. Claude Opus is genuinely better for hard problems. This is about:

**Economic sustainability**: Heavy users can't afford $100/mo API bills forever.

**Privacy**: Some code/data can't leave your infrastructure.

**Experimentation**: Try things without worrying about costs.

**Learning**: Run your own models, understand them, modify them.

**Resilience**: Don't depend on one vendor's uptime.

**Control**: Your tools, your rules.

## Implementation Priorities

For ttyclaude v0.2 (multi-provider support):

**Must have**:
- OpenAI-compatible provider type
- Provider connection status tracking
- Graceful fallback on provider failure

**Should have**:
- Provider auto-discovery (probe common ports)
- Connection retry logic
- Per-provider timeout configuration

**Nice to have**:
- SSH tunnel integration (connect to remote:8080 via tunnel)
- Provider health monitoring
- Usage statistics per provider

**Future**:
- Automatic model downloads (like ollama)
- Provider capability negotiation
- Load balancing across multiple instances

## Closing Thoughts

The VPS use case isn't exotic - it's practical. Lots of devs have underutilized VPS instances. Lots of devs want privacy and control. Small models on decent hardware are genuinely useful now.

ttyclaude embracing this pattern - treating self-hosted VPS models as first-class citizens alongside commercial APIs - makes it a tool for the long term. You can start with APIs, move to self-hosted as you grow, use both simultaneously, choose based on context.

That's the Unix philosophy: composable tools, user choice, no lock-in.

---

**Next Steps**:
1. Implement OpenAI-compatible provider type
2. Test with lmcli, ollama, vllm
3. Document VPS deployment
4. Create provider discovery mechanism
5. Build fallback logic

**Questions for the community**:
- What self-hosted tools do you use?
- What models work best for your VPS specs?
- What's your fallback strategy when primary provider is down?
- SSH tunnel integration - worth the complexity?

---

**Last Updated**: 2025-11-09  
**Author**: Threadpanic  
**Focus**: VPS self-hosted LLM deployment patterns


