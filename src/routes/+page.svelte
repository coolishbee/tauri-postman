<script lang="ts">
  import { onMount } from "svelte"
  import { commands } from "$lib/bindings"
  import { Store } from "@tauri-apps/plugin-store"
  import { Splitpanes, Pane } from "svelte-splitpanes"

  // Types
  interface HttpHeader {
    key: string
    value: string
    enabled: boolean
  }

  interface HttpRequest {
    method: string
    url: string
    headers: HttpHeader[]
    body: string
  }

  interface HttpResponse {
    statusCode: number
    status: string
    headers: Partial<Record<string, string>>
    body: string
    responseTimeMs: number
    error: string | null
  }

  interface Template {
    id: string
    name: string
    request: HttpRequest
    createdAt: string
    updatedAt: string
  }

  interface HistoryItem {
    id: string
    request: HttpRequest
    response: HttpResponse
    timestamp: string
  }

  // State
  let method = $state("GET")
  let url = $state("")
  let headers = $state<HttpHeader[]>([{ key: "Content-Type", value: "application/json", enabled: true }])
  let body = $state("")

  let response = $state<HttpResponse | null>(null)
  let isLoading = $state(false)
  let activeRequestTab = $state<"headers" | "body">("headers")
  let activeResponseTab = $state<"body" | "headers">("body")

  let templates = $state<Template[]>([])
  let history = $state<HistoryItem[]>([])
  let showSaveDialog = $state(false)
  let templateName = $state("")

  // Theme state
  let theme = $state<"system" | "dark" | "light">("system")

  const methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"]
  const MAX_HISTORY = 50

  let store: Store | null = null

  onMount(async () => {
    store = await Store.load("http-client.json")
    await loadData()
    await loadTheme()
    applyTheme(theme)

    // Listen for system theme changes
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)")
    mediaQuery.addEventListener("change", () => {
      if (theme === "system") {
        applyTheme("system")
      }
    })
  })

  async function loadData() {
    if (!store) return
    const savedTemplates = await store.get<Template[]>("templates")
    const savedHistory = await store.get<HistoryItem[]>("history")
    if (savedTemplates) templates = savedTemplates
    if (savedHistory) history = savedHistory
  }

  async function saveData() {
    if (!store) return
    await store.set("templates", templates)
    await store.set("history", history)
    await store.save()
  }

  async function loadTheme() {
    if (!store) return
    const savedTheme = await store.get<"system" | "dark" | "light">("theme")
    if (savedTheme) theme = savedTheme
  }

  async function saveTheme() {
    if (!store) return
    await store.set("theme", theme)
    await store.save()
  }

  function applyTheme(value: "system" | "dark" | "light") {
    const html = document.documentElement
    
    if (value === "system") {
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches
      html.classList.toggle("dark", prefersDark)
    } else if (value === "dark") {
      html.classList.add("dark")
    } else {
      html.classList.remove("dark")
    }
  }

  async function handleThemeChange() {
    applyTheme(theme)
    await saveTheme()
  }

  function addHeader() {
    headers = [...headers, { key: "", value: "", enabled: true }]
  }

  function removeHeader(index: number) {
    headers = headers.filter((_, i) => i !== index)
  }

  async function sendRequest() {
    if (!url.trim()) return

    isLoading = true
    response = null

    try {
      const request: HttpRequest = {
        method,
        url,
        headers: headers.filter(h => h.key.trim()),
        body
      }

      const result = await commands.sendHttpRequest(request)
      
      if (result.status === "ok") {
        response = JSON.parse(JSON.stringify(result.data))
        
        if (!result.data.error) {
          const historyItem: HistoryItem = {
            id: crypto.randomUUID(),
            request: JSON.parse(JSON.stringify(request)),
            response: JSON.parse(JSON.stringify(result.data)),
            timestamp: new Date().toISOString()
          }
          history = [historyItem, ...history].slice(0, MAX_HISTORY)
          await saveData()
        }
      } else {
        const errorMsg = typeof result.error === "string" 
          ? result.error 
          : Object.values(result.error)[0] as string
        response = {
          statusCode: 0,
          status: "Error",
          headers: {},
          body: "",
          responseTimeMs: 0,
          error: errorMsg
        }
      }
    } catch (e) {
      response = {
        statusCode: 0,
        status: "Error",
        headers: {},
        body: "",
        responseTimeMs: 0,
        error: String(e)
      }
    } finally {
      isLoading = false
    }
  }

  function loadRequest(request: HttpRequest) {
    method = request.method
    url = request.url
    headers = JSON.parse(JSON.stringify(request.headers))
    if (headers.length === 0) {
      headers = [{ key: "", value: "", enabled: true }]
    }
    body = request.body
    response = null
  }

  async function saveTemplate() {
    if (!templateName.trim()) return

    const existingIndex = templates.findIndex(t => t.name === templateName)
    const now = new Date().toISOString()

    const template: Template = {
      id: existingIndex >= 0 ? templates[existingIndex].id : crypto.randomUUID(),
      name: templateName,
      request: {
        method,
        url,
        headers: JSON.parse(JSON.stringify(headers)),
        body
      },
      createdAt: existingIndex >= 0 ? templates[existingIndex].createdAt : now,
      updatedAt: now
    }

    if (existingIndex >= 0) {
      templates[existingIndex] = template
    } else {
      templates = [...templates, template].sort((a, b) => a.name.localeCompare(b.name))
    }

    await saveData()
    showSaveDialog = false
    templateName = ""
  }

  async function deleteTemplate(id: string) {
    templates = templates.filter(t => t.id !== id)
    await saveData()
  }

  async function clearHistory() {
    history = []
    await saveData()
  }

  function getStatusColor(code: number): string {
    if (code >= 200 && code < 300) return "text-green-400"
    if (code >= 400) return "text-red-400"
    return "text-yellow-400"
  }

  function getStatusText(code: number): string {
    const texts: Record<number, string> = {
      200: "OK", 201: "Created", 204: "No Content",
      400: "Bad Request", 401: "Unauthorized", 403: "Forbidden",
      404: "Not Found", 500: "Server Error"
    }
    return texts[code] || ""
  }

  function formatUrl(urlStr: string, maxLen = 30): string {
    if (urlStr.length <= maxLen) return urlStr
    return urlStr.substring(0, maxLen) + "..."
  }
</script>

<div class="h-screen flex flex-col">
  <!-- Header Bar with Theme Selector -->
  <div class="h-12 border-b border-surface-500/20 flex items-center justify-between px-4 shrink-0">
    <h1 class="font-bold text-lg">Postman</h1>
    <div class="flex items-center gap-2">
      <label for="theme-select" class="text-sm opacity-70">Theme:</label>
      <select 
        id="theme-select"
        bind:value={theme}
        onchange={handleThemeChange}
        class="select select-sm preset-outlined-surface-500 w-28"
      >
        <option value="system">System</option>
        <option value="dark">Dark</option>
        <option value="light">Light</option>
      </select>
    </div>
  </div>

  <!-- Main Content with Splitpanes -->
  <div class="flex-1 min-h-0">
    <Splitpanes class="default-theme h-full">
      <!-- Sidebar Pane -->
      <Pane size={25} minSize={15} maxSize={40}>
        <Splitpanes horizontal class="h-full">
          <!-- Templates Section -->
          <Pane size={50} minSize={20}>
            <div class="h-full flex flex-col border-r border-surface-500/20">
              <div class="p-3 border-b border-surface-500/20 shrink-0">
                <h3 class="font-bold text-sm flex items-center gap-2">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                  </svg>
                  Templates
                </h3>
              </div>
              <div class="flex-1 overflow-y-auto p-2 space-y-1">
                {#if templates.length === 0}
                  <p class="text-sm text-surface-400 p-2">No templates saved</p>
                {:else}
                  {#each templates as template (template.id)}
                    <div class="group flex items-center gap-1 p-2 rounded hover:bg-surface-500/20 cursor-pointer"
                         role="button"
                         tabindex="0"
                         onclick={() => loadRequest(template.request)}
                         onkeydown={(e) => e.key === "Enter" && loadRequest(template.request)}>
                      <div class="flex-1 min-w-0">
                        <p class="font-medium text-sm truncate">{template.name}</p>
                        <p class="text-xs text-surface-400 truncate">{template.request.method} {formatUrl(template.request.url, 20)}</p>
                      </div>
                      <button class="opacity-0 group-hover:opacity-100 p-1 hover:bg-red-500/20 rounded"
                              aria-label="Delete template"
                              onclick={(e) => { e.stopPropagation(); deleteTemplate(template.id) }}>
                        <svg class="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>
                    </div>
                  {/each}
                {/if}
              </div>
              <div class="p-2 border-t border-surface-500/20 shrink-0">
                <button class="w-full btn btn-sm preset-filled-primary-500"
                        onclick={() => showSaveDialog = true}>
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                  </svg>
                  Save Current
                </button>
              </div>
            </div>
          </Pane>

          <!-- History Section -->
          <Pane size={50} minSize={20}>
            <div class="h-full flex flex-col border-r border-surface-500/20">
              <div class="p-3 border-b border-surface-500/20 shrink-0">
                <h3 class="font-bold text-sm flex items-center gap-2">
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  History
                </h3>
              </div>
              <div class="flex-1 overflow-y-auto p-2 space-y-1">
                {#if history.length === 0}
                  <p class="text-sm text-surface-400 p-2">No history yet</p>
                {:else}
                  {#each history as item (item.id)}
                    <div class="p-2 rounded hover:bg-surface-500/20 cursor-pointer"
                         role="button"
                         tabindex="0"
                         onclick={() => loadRequest(item.request)}
                         onkeydown={(e) => e.key === "Enter" && loadRequest(item.request)}>
                      <div class="flex items-center gap-2">
                        <span class="font-mono text-xs font-bold">{item.request.method}</span>
                        <span class="text-xs truncate flex-1">{formatUrl(item.request.url, 18)}</span>
                      </div>
                      <div class="flex items-center gap-2 mt-1">
                        <span class="text-xs {getStatusColor(item.response.statusCode)}">
                          {item.response.statusCode} {getStatusText(item.response.statusCode)}
                        </span>
                        <span class="text-xs text-surface-400">{item.response.responseTimeMs}ms</span>
                      </div>
                    </div>
                  {/each}
                {/if}
              </div>
              {#if history.length > 0}
                <div class="p-2 border-t border-surface-500/20 shrink-0">
                  <button class="w-full btn btn-sm preset-outlined-error-500"
                          onclick={clearHistory}>
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                    Clear All
                  </button>
                </div>
              {/if}
            </div>
          </Pane>
        </Splitpanes>
      </Pane>

      <!-- Main Content Pane -->
      <Pane size={75} minSize={50}>
        <div class="h-full flex flex-col">
          <!-- URL Bar -->
          <div class="flex gap-2 p-4 shrink-0">
            <select bind:value={method}
                    class="select w-28 preset-outlined-surface-500">
              {#each methods as m}
                <option value={m}>{m}</option>
              {/each}
            </select>
            <input type="text" bind:value={url}
                   placeholder="Enter URL (e.g., https://api.example.com/users)"
                   class="input flex-1 preset-outlined-surface-500"
                   onkeydown={(e) => e.key === "Enter" && sendRequest()} />
            <button class="btn preset-filled-primary-500"
                    onclick={sendRequest}
                    disabled={isLoading || !url.trim()}>
              {#if isLoading}
                <svg class="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
              {:else}
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                Send
              {/if}
            </button>
          </div>

          <!-- Request/Response Split -->
          <div class="flex-1 min-h-0 px-4 pb-4">
            <Splitpanes horizontal class="h-full">
              <!-- Request Panel -->
              <Pane size={50} minSize={20}>
                <div class="h-full flex flex-col card preset-outlined-surface-500">
                  <div class="p-3 border-b border-surface-500/20 shrink-0">
                    <h3 class="font-bold">Request</h3>
                  </div>
                  <div class="flex border-b border-surface-500/20 shrink-0">
                    <button class="px-4 py-2 text-sm font-medium {activeRequestTab === 'headers' ? 'border-b-2 border-primary-500 text-primary-500' : 'text-surface-400'}"
                            onclick={() => activeRequestTab = 'headers'}>
                      Headers ({headers.filter(h => h.key.trim()).length})
                    </button>
                    <button class="px-4 py-2 text-sm font-medium {activeRequestTab === 'body' ? 'border-b-2 border-primary-500 text-primary-500' : 'text-surface-400'}"
                            onclick={() => activeRequestTab = 'body'}>
                      Body
                    </button>
                  </div>
                  <div class="flex-1 overflow-auto p-3">
                    {#if activeRequestTab === 'headers'}
                      <div class="space-y-2">
                        {#each headers as header, i}
                          <div class="flex items-center gap-2">
                            <input type="checkbox" bind:checked={header.enabled}
                                   class="checkbox preset-filled-surface-500" />
                            <input type="text" bind:value={header.key}
                                   placeholder="Header name"
                                   class="input flex-1 input-sm preset-outlined-surface-500" />
                            <input type="text" bind:value={header.value}
                                   placeholder="Header value"
                                   class="input flex-1 input-sm preset-outlined-surface-500" />
                            <button class="btn btn-sm preset-ghost"
                                    aria-label="Remove header"
                                    onclick={() => removeHeader(i)}>
                              <svg class="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                              </svg>
                            </button>
                          </div>
                        {/each}
                        <button class="btn btn-sm preset-ghost"
                                onclick={addHeader}>
                          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                          </svg>
                          Add Header
                        </button>
                      </div>
                    {:else}
                      <textarea bind:value={body}
                                placeholder="Request body (JSON)"
                                class="textarea w-full h-full font-mono text-sm preset-outlined-surface-500 resize-none"></textarea>
                    {/if}
                  </div>
                </div>
              </Pane>

              <!-- Response Panel -->
              <Pane size={50} minSize={20}>
                <div class="h-full flex flex-col card preset-outlined-surface-500">
                  <div class="p-3 border-b border-surface-500/20 flex items-center gap-4 shrink-0">
                    <h3 class="font-bold">Response</h3>
                    {#if response}
                      {#if response.error}
                        <span class="text-sm text-red-400">Error: {response.error}</span>
                      {:else}
                        <span class="text-sm {getStatusColor(response.statusCode)}">
                          Status: {response.status}
                        </span>
                        <span class="text-sm text-surface-400">
                          Time: {response.responseTimeMs}ms
                        </span>
                      {/if}
                    {:else}
                      <span class="text-sm text-surface-400">Status: -</span>
                      <span class="text-sm text-surface-400">Time: -</span>
                    {/if}
                  </div>
                  <div class="flex border-b border-surface-500/20 shrink-0">
                    <button class="px-4 py-2 text-sm font-medium {activeResponseTab === 'body' ? 'border-b-2 border-primary-500 text-primary-500' : 'text-surface-400'}"
                            onclick={() => activeResponseTab = 'body'}>
                      Body
                    </button>
                    <button class="px-4 py-2 text-sm font-medium {activeResponseTab === 'headers' ? 'border-b-2 border-primary-500 text-primary-500' : 'text-surface-400'}"
                            onclick={() => activeResponseTab = 'headers'}>
                      Headers
                    </button>
                  </div>
                  <div class="flex-1 overflow-auto p-3">
                    {#if response && !response.error}
                      {#if activeResponseTab === 'body'}
                        <pre class="font-mono text-sm whitespace-pre-wrap break-all">{response.body || "(empty)"}</pre>
                      {:else}
                        <div class="space-y-1">
                          {#each Object.entries(response.headers) as [key, value]}
                            <div class="font-mono text-sm">
                              <span class="text-primary-400">{key}:</span> {value}
                            </div>
                          {/each}
                        </div>
                      {/if}
                    {:else if response?.error}
                      <p class="text-red-400">{response.error}</p>
                    {:else}
                      <p class="text-surface-400">Response will appear here</p>
                    {/if}
                  </div>
                </div>
              </Pane>
            </Splitpanes>
          </div>
        </div>
      </Pane>
    </Splitpanes>
  </div>
</div>

<!-- Save Template Dialog -->
{#if showSaveDialog}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
       role="dialog"
       aria-modal="true"
       tabindex="-1"
       onclick={() => showSaveDialog = false}
       onkeydown={(e) => e.key === "Escape" && (showSaveDialog = false)}>
    <div class="card bg-surface-200 dark:bg-surface-800 p-4 w-80"
         onclick={(e) => e.stopPropagation()}
         onkeydown={(e) => e.stopPropagation()}>
      <h3 class="font-bold mb-4">Save as Template</h3>
      <input type="text" bind:value={templateName}
             placeholder="Enter template name"
             class="input w-full preset-outlined-surface-500 mb-4"
             onkeydown={(e) => e.key === "Enter" && saveTemplate()} />
      <div class="flex justify-end gap-2">
        <button class="btn preset-ghost"
                onclick={() => showSaveDialog = false}>
          Cancel
        </button>
        <button class="btn preset-filled-primary-500"
                onclick={saveTemplate}
                disabled={!templateName.trim()}>
          Save
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(.splitpanes.default-theme .splitpanes__pane) {
    background-color: var(--body-background-color);
  }
  :global(html.dark .splitpanes.default-theme .splitpanes__pane) {
    background-color: var(--body-background-color-dark);
  }
  :global(.splitpanes.default-theme .splitpanes__splitter) {
    background-color: rgb(var(--color-surface-500) / 0.2);
    position: relative;
  }
  :global(.splitpanes.default-theme .splitpanes__splitter:before) {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    transition: opacity 0.3s;
    background-color: rgb(var(--color-primary-500) / 0.3);
    opacity: 0;
    z-index: 1;
  }
  :global(.splitpanes.default-theme .splitpanes__splitter:hover:before) {
    opacity: 1;
  }
  :global(.splitpanes--vertical > .splitpanes__splitter:before) {
    left: -3px;
    right: -3px;
    height: 100%;
    width: auto;
  }
  :global(.splitpanes--horizontal > .splitpanes__splitter:before) {
    top: -3px;
    bottom: -3px;
    width: 100%;
    height: auto;
  }
</style>
