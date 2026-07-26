import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it } from 'vitest';

import { useDialogFocus } from './useDialogFocus';

const Fixture = () => {
  const [open, setOpen] = useState(false);
  const dialogRef = useDialogFocus<HTMLElement>(open, () => setOpen(false));

  return (
    <>
      <button type="button" onClick={() => setOpen(true)}>
        Open
      </button>
      {open ? (
        <section ref={dialogRef} role="dialog" aria-label="Example" tabIndex={-1}>
          <button type="button">First</button>
          <button type="button" onClick={() => setOpen(false)}>
            Last
          </button>
        </section>
      ) : null}
    </>
  );
};

describe('useDialogFocus', () => {
  it('moves, contains, closes, and restores keyboard focus', async () => {
    const user = userEvent.setup();
    render(<Fixture />);

    const trigger = screen.getByRole('button', { name: 'Open' });
    await user.click(trigger);
    const first = screen.getByRole('button', { name: 'First' });
    const last = screen.getByRole('button', { name: 'Last' });
    expect(first).toHaveFocus();

    last.focus();
    await user.tab();
    expect(first).toHaveFocus();

    await user.keyboard('{Escape}');
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
  });
});
