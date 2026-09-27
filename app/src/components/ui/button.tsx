import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "cn";
import { Slot } from "radix-ui";
import type * as React from "react";

/** NSButton look-alikes: bordered push button, prominent (default) button and borderless icon button. */
const buttonVariants = cva(
  "inline-flex shrink-0 items-center justify-center gap-1 rounded-[6px] whitespace-nowrap transition-[background-color,filter] duration-100 outline-none disabled:pointer-events-none disabled:opacity-40 [&_svg]:pointer-events-none [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        prominent: "bg-accent font-medium text-white shadow-[0_0.5px_1px_rgb(0_0_0/0.2)] active:brightness-90",
        bordered:
          "bg-control text-label shadow-[0_0_0_0.5px_var(--control-edge),0_0.5px_1.5px_rgb(0_0_0/0.1)] active:bg-pressed",
        borderless: "text-secondary-label hover:bg-hover hover:text-label active:bg-pressed",
      },
      size: {
        regular: "h-[22px] px-2.5 text-[13px]",
        small: "h-[19px] min-w-[52px] px-2 text-[11px]",
        icon: "size-[22px] [&_svg]:size-[14px]",
      },
    },
    defaultVariants: { variant: "bordered", size: "regular" },
  },
);

type ButtonProps = React.ComponentProps<"button"> & VariantProps<typeof buttonVariants> & { asChild?: boolean };

function Button({ className, variant, size, asChild = false, ...props }: ButtonProps) {
  const Comp = asChild ? Slot.Root : "button";
  return <Comp data-slot="button" className={cn(buttonVariants({ variant, size, className }))} {...props} />;
}

export { Button };
