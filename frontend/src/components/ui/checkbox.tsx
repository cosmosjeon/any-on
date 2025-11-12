import * as React from 'react';
import { Check } from 'lucide-react';
import { cn } from '@/lib/utils';

interface CheckboxProps {
  id?: string;
  checked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  className?: string;
  disabled?: boolean;
}

const Checkbox = React.forwardRef<HTMLButtonElement, CheckboxProps>(
  (
    { className, checked = false, onCheckedChange, disabled, ...props },
    ref
  ) => {
    return (
      <button
        type="button"
        role="checkbox"
        aria-checked={checked}
        ref={ref}
        className={cn(
          'peer h-4 w-4 shrink-0 rounded-sm border-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
          checked
            ? 'bg-primary border-primary text-primary-foreground'
            : 'border-input bg-background hover:bg-accent hover:border-accent-foreground/20',
          className
        )}
        disabled={disabled}
        onClick={() => onCheckedChange?.(!checked)}
        {...props}
      >
        {checked && (
          <div className="flex items-center justify-center text-current">
            <Check className="h-3.5 w-3.5 stroke-[3]" />
          </div>
        )}
      </button>
    );
  }
);
Checkbox.displayName = 'Checkbox';

export { Checkbox };
