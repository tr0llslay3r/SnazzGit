<script lang="ts">
  import { showCredentialDialog, pendingCredentialRequest, addToast } from '$lib/stores/ui';
  import { repoInfo, refreshAll } from '$lib/stores/repo';
  import * as tauri from '$lib/utils/tauri';

  let username = $state('');
  let password = $state('');
  let saveCredentials = $state(true);
  let isSubmitting = $state(false);
  let inputEl: HTMLInputElement = $state(null!);

  $effect(() => {
    if ($showCredentialDialog && inputEl) {
      inputEl.focus();
    }
  });

  function isAuthError(e: unknown): boolean {
    return String(e).includes('Authentication failed');
  }

  async function submit() {
    const request = $pendingCredentialRequest;
    if (!request || !username.trim()) return;
    isSubmitting = true;

    const creds = { username: username.trim(), password };

    try {
      if (request.operation === 'clone' && request.url && request.clonePath) {
        const workdir = await tauri.cloneRepo(request.url, request.clonePath, creds);
        if (saveCredentials) {
          await tauri.storeCredentials(request.url, creds.username, creds.password).catch(() => {});
        }
        const info = await tauri.openRepository(workdir);
        $repoInfo = info;
        await tauri.addRecentRepo(info.path, info.name);
        await refreshAll();
        addToast(`Cloned ${info.name}`, 'success');
      } else if ($repoInfo) {
        const remoteName = request.remoteName || $repoInfo.remotes[0];
        const remoteUrl = request.url || '';

        if (request.operation === 'fetch') {
          await tauri.fetchRemote($repoInfo.path, remoteName, creds);
        } else if (request.operation === 'pull') {
          await tauri.pull($repoInfo.path, remoteName, creds);
        } else if (request.operation === 'push') {
          await tauri.push($repoInfo.path, remoteName, creds);
        }

        if (saveCredentials && remoteUrl) {
          await tauri.storeCredentials(remoteUrl, creds.username, creds.password).catch(() => {});
        }

        await refreshAll();
        addToast(`${request.operation.charAt(0).toUpperCase() + request.operation.slice(1)} complete`, 'success');
      }

      close();
    } catch (e) {
      if (isAuthError(e)) {
        addToast('Authentication failed. Check your credentials.', 'error');
      } else {
        addToast(`${request.operation} failed: ${e}`, 'error');
        close();
      }
    } finally {
      isSubmitting = false;
    }
  }

  function close() {
    username = '';
    password = '';
    saveCredentials = true;
    isSubmitting = false;
    $showCredentialDialog = false;
    $pendingCredentialRequest = null;
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !isSubmitting) submit();
    if (e.key === 'Escape') close();
  }

  let subtitle = $derived(() => {
    const req = $pendingCredentialRequest;
    if (!req) return '';
    if (req.url) return req.url;
    if (req.remoteName) return req.remoteName;
    return '';
  });
</script>

{#if $showCredentialDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="dialog-overlay" onclick={close}>
    <!-- svelte-ignore a11y_interactive_supports_focus, a11y_no_static_element_interactions -->
    <div class="dialog" onclick={(e) => e.stopPropagation()} onkeydown={onKeydown} role="dialog" tabindex="-1">
      <h3 class="dialog-title">Authentication Required</h3>
      {#if subtitle()}
        <p class="dialog-subtitle">{subtitle()}</p>
      {/if}
      <div class="dialog-fields">
        <input
          class="dialog-input"
          type="text"
          placeholder="Username"
          bind:this={inputEl}
          bind:value={username}
          disabled={isSubmitting}
        />
        <input
          class="dialog-input"
          type="password"
          placeholder="Password / Token"
          bind:value={password}
          disabled={isSubmitting}
        />
        <label class="dialog-checkbox">
          <input type="checkbox" bind:checked={saveCredentials} disabled={isSubmitting} />
          <span>Save credentials to keychain</span>
        </label>
      </div>
      <div class="dialog-actions">
        <button class="btn-secondary" onclick={close} disabled={isSubmitting}>Cancel</button>
        <button class="btn-primary" onclick={submit} disabled={!username.trim() || isSubmitting}>
          {isSubmitting ? 'Authenticating...' : 'Authenticate'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .dialog {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px;
    width: 400px;
    max-width: 90vw;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  }
  .dialog-title {
    margin: 0 0 4px;
    font-size: 16px;
    color: var(--text-primary);
  }
  .dialog-subtitle {
    margin: 0 0 16px;
    font-size: 12px;
    color: var(--text-secondary);
    word-break: break-all;
  }
  .dialog-fields {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .dialog-input {
    width: 100%;
    padding: 8px 12px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 14px;
    outline: none;
    box-sizing: border-box;
  }
  .dialog-input:focus {
    border-color: var(--accent);
  }
  .dialog-input:disabled {
    opacity: 0.6;
  }
  .dialog-checkbox {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--text-secondary);
    cursor: pointer;
  }
  .dialog-checkbox input[type="checkbox"] {
    accent-color: var(--accent);
  }
  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
  .btn-secondary {
    padding: 6px 16px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 13px;
  }
  .btn-primary {
    padding: 6px 16px;
    border: none;
    border-radius: 6px;
    background: var(--accent);
    color: var(--bg-primary);
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
  }
  .btn-primary:disabled {
    opacity: 0.4;
  }
</style>
