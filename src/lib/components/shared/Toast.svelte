<script lang="ts">
  import { toasts, removeToast } from '$lib/stores/ui';
</script>

<div class="toast-container">
  {#each $toasts as toast (toast.id)}
    <div class="toast toast-{toast.type}">
      <span class="toast-message">{toast.message}</span>
      <button class="toast-close" onclick={() => removeToast(toast.id)} aria-label="Dismiss">
        <svg viewBox="0 0 16 16" width="12" height="12">
          <path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06z" fill="currentColor"/>
        </svg>
      </button>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: 40px;
    right: 16px;
    z-index: 200;
    display: flex;
    flex-direction: column-reverse;
    gap: 8px;
  }
  .toast {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    border-radius: 8px;
    font-size: 13px;
    background: var(--bg-surface);
    color: var(--text-primary);
    border: 1px solid var(--border);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    animation: slide-in 0.2s ease-out;
    max-width: 400px;
  }
  .toast-success { border-left: 3px solid var(--success); }
  .toast-error { border-left: 3px solid var(--danger); }
  .toast-warning { border-left: 3px solid var(--warning); }
  .toast-info { border-left: 3px solid var(--accent); }
  .toast-message { flex: 1; }
  .toast-close {
    display: flex;
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 2px;
  }
  @keyframes slide-in {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }
</style>
