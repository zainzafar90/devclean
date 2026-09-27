import { cn } from "cn";
import { Switch as SwitchPrimitive } from "radix-ui";
import type * as React from "react";

/** NSSwitch at small control size: 32 × 18 track, white knob, accent when on. */
function Switch({ className, ...props }: React.ComponentProps<typeof SwitchPrimitive.Root>) {
  return (
    <SwitchPrimitive.Root
      data-slot="switch"
      className={cn(
        "inline-flex h-[18px] w-[32px] shrink-0 items-center rounded-full p-px transition-colors duration-150 outline-none data-checked:bg-accent data-unchecked:bg-track data-unchecked:shadow-[inset_0_0_0_0.5px_var(--separator)] data-disabled:opacity-50",
        className,
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb className="block size-4 rounded-full bg-white shadow-[0_0_0_0.5px_rgb(0_0_0/0.08),0_1px_2px_rgb(0_0_0/0.3)] transition-transform duration-150 data-checked:translate-x-[14px]" />
    </SwitchPrimitive.Root>
  );
}

export { Switch };
