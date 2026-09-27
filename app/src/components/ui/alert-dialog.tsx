import { cn } from "cn";
import { AlertDialog as AlertDialogPrimitive } from "radix-ui";
import type * as React from "react";
import { Button } from "@/components/ui/button";

/** NSAlert as macOS 26 draws it: centred icon, bold title, small message, two equal-width buttons. */
const AlertDialog = AlertDialogPrimitive.Root;
const AlertDialogTrigger = AlertDialogPrimitive.Trigger;

function AlertDialogContent({ className, ...props }: React.ComponentProps<typeof AlertDialogPrimitive.Content>) {
  return (
    <AlertDialogPrimitive.Portal>
      <AlertDialogPrimitive.Overlay className="fixed inset-0 z-50 bg-black/5 data-open:animate-in data-open:fade-in-0 data-closed:animate-out data-closed:fade-out-0" />
      <AlertDialogPrimitive.Content
        className={cn(
          "fixed top-1/2 left-1/2 z-50 flex w-[260px] -translate-x-1/2 -translate-y-1/2 flex-col items-center gap-2.5 rounded-[14px] bg-sheet p-4 text-center shadow-[0_0_0_0.5px_var(--separator),0_12px_32px_rgb(0_0_0/0.28)] backdrop-blur-2xl outline-none data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95 data-closed:animate-out data-closed:fade-out-0",
          className,
        )}
        {...props}
      />
    </AlertDialogPrimitive.Portal>
  );
}

function AlertDialogTitle({ className, ...props }: React.ComponentProps<typeof AlertDialogPrimitive.Title>) {
  return <AlertDialogPrimitive.Title className={cn("text-[13px] font-bold", className)} {...props} />;
}

function AlertDialogDescription({ className, ...props }: React.ComponentProps<typeof AlertDialogPrimitive.Description>) {
  return (
    <AlertDialogPrimitive.Description className={cn("text-[11px] leading-[14px] text-label", className)} {...props} />
  );
}

function AlertDialogFooter({ className, ...props }: React.ComponentProps<"div">) {
  return <div className={cn("mt-1 grid w-full grid-cols-2 gap-2", className)} {...props} />;
}

function AlertDialogCancel(props: React.ComponentProps<typeof AlertDialogPrimitive.Cancel>) {
  return (
    <Button variant="bordered" asChild>
      <AlertDialogPrimitive.Cancel {...props} />
    </Button>
  );
}

function AlertDialogAction(props: React.ComponentProps<typeof AlertDialogPrimitive.Action>) {
  return (
    <Button variant="prominent" asChild>
      <AlertDialogPrimitive.Action {...props} />
    </Button>
  );
}

export {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogTitle,
  AlertDialogTrigger,
};
