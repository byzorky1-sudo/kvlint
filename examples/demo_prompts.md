# Example Bad Prompts (Hazardous for Prefix KV-Cache)

## 1. Dynamic Timestamp in Header (Rule KV001)
```python
# BAD: Changes the hash of the first token on every call -> 0% prefix cache hit rate
system_prompt = f"""
Current timestamp: {datetime.now()}
You are a customer support agent.
Instructions:
- Be concise and helpful.
- Resolve user tickets accurately.
"""
```

## 2. Dynamic UUID / Session ID at Top (Rule KV002)
```typescript
// BAD: Random UUID at prompt header destroys cross-turn KV reuse
const systemMessage = `
Session ID: ${crypto.randomUUID()}
User permissions: read, write
Tool catalog:
- search_kb
- update_ticket
`;
```

## 3. Dynamic User Variables Before Static Instructions (Rule KV003)
```jinja2
{# BAD: User query placed before static tool definitions breaks shared prefix tree #}
User Query: {{ user_input }}

System Guidelines:
You are an expert SQL engineer.
Follow these formatting rules:
- Always use ANSI SQL syntax.
- Quote identifier names properly.
```

---

# Example Good Prompts (Optimized for 90%+ Cache Hits)

```python
# GOOD: Static system instructions come first, timestamps quantized or at the end
SYSTEM_PROMPT = """
You are a customer support agent.
Instructions:
- Be concise and helpful.
- Resolve user tickets accurately.
"""

# User message handles dynamic parameters:
user_message = f"User query: {query}\n[Current date: {today_date}]"
```
