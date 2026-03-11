import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { toasts, addToast, removeToast } from '$lib/stores/ui';

beforeEach(() => {
  toasts.set([]);
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('addToast', () => {
  it('adds a toast to the store', () => {
    addToast('Hello');
    expect(get(toasts)).toHaveLength(1);
  });

  it('sets the message correctly', () => {
    addToast('Test message');
    expect(get(toasts)[0].message).toBe('Test message');
  });

  it('defaults type to info', () => {
    addToast('Info toast');
    expect(get(toasts)[0].type).toBe('info');
  });

  it('accepts explicit type: success', () => {
    addToast('Done!', 'success');
    expect(get(toasts)[0].type).toBe('success');
  });

  it('accepts explicit type: error', () => {
    addToast('Oops', 'error');
    expect(get(toasts)[0].type).toBe('error');
  });

  it('accepts explicit type: warning', () => {
    addToast('Watch out', 'warning');
    expect(get(toasts)[0].type).toBe('warning');
  });

  it('assigns a truthy id to each toast', () => {
    addToast('First');
    expect(get(toasts)[0].id).toBeTruthy();
  });

  it('assigns unique ids to multiple toasts', () => {
    addToast('First');
    addToast('Second');
    const [a, b] = get(toasts);
    expect(a.id).not.toBe(b.id);
  });

  it('appends multiple toasts in order', () => {
    addToast('First');
    addToast('Second');
    addToast('Third');
    const messages = get(toasts).map((t) => t.message);
    expect(messages).toEqual(['First', 'Second', 'Third']);
  });

  it('auto-removes the toast after the given timeout', () => {
    addToast('Temporary', 'info', 1000);
    expect(get(toasts)).toHaveLength(1);
    vi.advanceTimersByTime(1000);
    expect(get(toasts)).toHaveLength(0);
  });

  it('does not auto-remove before the timeout elapses', () => {
    addToast('Still here', 'info', 2000);
    vi.advanceTimersByTime(1999);
    expect(get(toasts)).toHaveLength(1);
  });

  it('does not auto-remove when timeout is 0', () => {
    addToast('Permanent', 'info', 0);
    vi.advanceTimersByTime(60_000);
    expect(get(toasts)).toHaveLength(1);
  });

  it('stores the timeout value on the toast', () => {
    addToast('Timed', 'info', 3000);
    expect(get(toasts)[0].timeout).toBe(3000);
  });
});

describe('removeToast', () => {
  it('removes a toast by id', () => {
    addToast('To remove');
    const id = get(toasts)[0].id;
    removeToast(id);
    expect(get(toasts)).toHaveLength(0);
  });

  it('only removes the toast with the matching id', () => {
    addToast('Keep me');
    addToast('Remove me');
    const removeId = get(toasts)[1].id;
    removeToast(removeId);
    const remaining = get(toasts);
    expect(remaining).toHaveLength(1);
    expect(remaining[0].message).toBe('Keep me');
  });

  it('does nothing when the id does not exist', () => {
    addToast('Present');
    removeToast('non-existent-id');
    expect(get(toasts)).toHaveLength(1);
  });

  it('can remove the first of several toasts', () => {
    addToast('A');
    addToast('B');
    addToast('C');
    const firstId = get(toasts)[0].id;
    removeToast(firstId);
    const messages = get(toasts).map((t) => t.message);
    expect(messages).toEqual(['B', 'C']);
  });
});
