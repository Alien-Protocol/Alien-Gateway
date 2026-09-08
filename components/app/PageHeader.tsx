export function PageHeader({
  kicker,
  title,
  description,
}: {
  kicker: string;
  title: string;
  description?: string;
}) {
  return (
    <div className="app-fade min-w-0">
      <p className="font-raj text-xs font-semibold uppercase tracking-[0.22em] text-white/50 sm:text-sm sm:tracking-[0.28em]">
        {kicker}
      </p>
      <h1 className="mt-2 font-orbitron text-[1.35rem] font-extrabold uppercase tracking-[0.1em] sm:text-3xl sm:tracking-[0.12em]">
        {title}
      </h1>
      {description ? (
        <p className="mt-3 max-w-3xl font-exo text-sm leading-relaxed text-white/60 sm:text-base">
          {description}
        </p>
      ) : null}
    </div>
  );
}
