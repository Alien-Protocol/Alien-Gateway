import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";
import { Slot } from "radix-ui";

const buttonVariants = cva(
  "group/button inline-flex shrink-0 items-center justify-center gap-2 rounded-none border border-transparent font-orbitron text-xs font-extrabold tracking-[0.14em] uppercase whitespace-nowrap transition-all outline-none select-none focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-35 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
  {
    variants: {
      variant: {
        default:
          "relative overflow-hidden bg-primary text-primary-foreground border-primary hover:bg-white/90 animate-[btn-pulse_2.5s_ease-in-out_infinite] after:pointer-events-none after:absolute after:inset-0 after:bg-[linear-gradient(90deg,transparent_0%,rgba(255,255,255,0.35)_50%,transparent_100%)] after:translate-x-[-100%] after:transition-transform after:duration-500 hover:after:translate-x-full",
        outline:
          "border-white/30 bg-transparent text-white/70 hover:border-white hover:bg-white hover:text-black",
        secondary:
          "border-white/20 bg-white/5 text-white hover:bg-white/10",
        ghost:
          "border-transparent bg-transparent text-white/70 hover:bg-white/10 hover:text-white",
        destructive:
          "border-white/65 bg-transparent text-white hover:bg-white hover:text-black",
        link: "border-transparent text-white underline-offset-4 hover:underline tracking-normal font-exo font-medium normal-case",
      },
      size: {
        default: "h-10 px-4 py-[11px]",
        xs: "h-7 px-2 text-[10px]",
        sm: "h-8 px-3 text-[11px]",
        lg: "h-12 px-6 text-[13px]",
        icon: "size-10",
        "icon-xs": "size-6",
        "icon-sm": "size-8",
        "icon-lg": "size-11",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
);

function Button({
  className,
  variant = "default",
  size = "default",
  asChild = false,
  ...props
}: React.ComponentProps<"button"> &
  VariantProps<typeof buttonVariants> & {
    asChild?: boolean;
  }) {
  const Comp = asChild ? Slot.Root : "button";

  return (
    <Comp
      data-slot="button"
      data-variant={variant}
      data-size={size}
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    />
  );
}

export { Button, buttonVariants };
