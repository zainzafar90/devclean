import type { ReactNode } from "react";

interface SectionProps {
  title: string;
  detail?: ReactNode;
  trailing?: ReactNode;
  footer?: ReactNode;
  children: ReactNode;
}

/** A System Settings group: semibold header, inset translucent rounded list, optional footnote. */
export function Section({ title, detail, trailing, footer, children }: SectionProps) {
  return (
    <section className="flex flex-col gap-1.5">
      <header className="flex min-h-[19px] items-center gap-1.5 px-2">
        <h2 className="text-[12px] font-semibold text-label">{title}</h2>
        <span className="flex flex-1 items-center gap-1 text-[11px] text-secondary-label tabular-nums">{detail}</span>
        {trailing}
      </header>
      <ul className="flex flex-col rounded-[10px] bg-group px-2.5">{children}</ul>
      {footer ? <div className="px-2 text-[11px] leading-[14px] text-secondary-label">{footer}</div> : null}
    </section>
  );
}

interface RowProps {
  icon?: ReactNode;
  title: ReactNode;
  detail?: ReactNode;
  trailing?: ReactNode;
  children?: ReactNode;
}

export function Row({ icon, title, detail, trailing, children }: RowProps) {
  return (
    <li className="flex flex-col gap-1.5 border-separator py-[7px] not-first:border-t-[0.5px]">
      <div className="flex items-center gap-2">
        {icon}
        <div className="flex min-w-0 flex-1 flex-col gap-px">
          <span className="truncate text-[13px] text-label">{title}</span>
          {detail ? <span className="flex min-w-0 text-[11px] leading-[14px] text-secondary-label">{detail}</span> : null}
        </div>
        {trailing}
      </div>
      {children ? <div className="pl-[30px]">{children}</div> : null}
    </li>
  );
}
