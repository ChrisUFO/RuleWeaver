import * as React from "react";
import { cn } from "@/lib/utils";
import { Check } from "lucide-react";

interface CheckboxProps extends Omit<
  React.InputHTMLAttributes<HTMLInputElement>,
  "type" | "onChange"
> {
  checked: boolean;
  onChange: (checked: boolean) => void;
  indeterminate?: boolean;
}

const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  ({ className, checked, onChange, indeterminate, ...props }, ref) => {
    const innerRef = React.useRef<HTMLInputElement>(null);
    React.useImperativeHandle(ref, () => innerRef.current!);

    React.useEffect(() => {
      if (innerRef.current) {
        innerRef.current.indeterminate = indeterminate ?? false;
      }
    }, [indeterminate]);

    return (
      <label className="inline-flex items-center cursor-pointer">
        <span className="relative flex items-center justify-center">
          <input
            ref={innerRef}
            type="checkbox"
            checked={checked}
            onChange={(e) => onChange(e.target.checked)}
            className="sr-only peer"
            {...props}
          />
          <span
            className={cn(
              "h-4 w-4 shrink-0 rounded-sm border border-primary shadow focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
              "peer-checked:bg-primary peer-checked:text-primary-foreground",
              "peer-indeterminate:bg-primary peer-indeterminate:text-primary-foreground",
              className
            )}
          >
            {checked && !indeterminate && <Check className="h-3 w-3" aria-hidden="true" />}
            {indeterminate && <span className="h-[2px] w-2.5 bg-current block mx-auto" />}
          </span>
        </span>
      </label>
    );
  }
);
Checkbox.displayName = "Checkbox";

export { Checkbox };
