import { cn } from "@/lib/utils";

function Skeleton({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("animate-pulse rounded-md bg-muted", className)} {...props} />;
}

function CardSkeleton() {
  return (
    <div className="rounded-xl border bg-card p-6">
      <div className="flex items-center justify-between mb-4">
        <Skeleton className="h-4 w-24" />
        <Skeleton className="h-4 w-4 rounded-full" />
      </div>
      <Skeleton className="h-8 w-16" />
    </div>
  );
}

function RuleCardSkeleton() {
  return (
    <div className="rounded-xl border bg-card p-4">
      <div className="flex items-center gap-4">
        <Skeleton className="h-5 w-9 rounded-full" />
        <div className="flex-1 space-y-2">
          <div className="flex items-center gap-2">
            <Skeleton className="h-5 w-40" />
            <Skeleton className="h-5 w-16 rounded-full" />
          </div>
          <Skeleton className="h-4 w-full" />
          <div className="flex gap-1">
            <Skeleton className="h-5 w-16 rounded-full" />
            <Skeleton className="h-5 w-16 rounded-full" />
          </div>
        </div>
        <Skeleton className="h-9 w-9 rounded-md" />
      </div>
    </div>
  );
}

function DashboardSkeleton() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="space-y-2">
          <Skeleton className="h-8 w-32" />
          <Skeleton className="h-4 w-64" />
        </div>
        <div className="flex gap-2">
          <Skeleton className="h-9 w-24" />
          <Skeleton className="h-9 w-24" />
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <CardSkeleton />
        <CardSkeleton />
        <CardSkeleton />
        <CardSkeleton />
      </div>

      <div className="rounded-xl border bg-card p-6 space-y-4">
        <Skeleton className="h-6 w-40" />
        <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-4">
          <Skeleton className="h-16 rounded-md" />
          <Skeleton className="h-16 rounded-md" />
          <Skeleton className="h-16 rounded-md" />
          <Skeleton className="h-16 rounded-md" />
        </div>
      </div>
    </div>
  );
}

function RulesListSkeleton() {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <Skeleton className="h-8 w-16" />
        <Skeleton className="h-9 w-24" />
      </div>

      <div className="flex items-center gap-4">
        <Skeleton className="h-9 w-64" />
        <div className="flex gap-2">
          <Skeleton className="h-8 w-12" />
          <Skeleton className="h-8 w-16" />
          <Skeleton className="h-8 w-14" />
        </div>
      </div>

      <div className="space-y-3">
        <RuleCardSkeleton />
        <RuleCardSkeleton />
        <RuleCardSkeleton />
      </div>
    </div>
  );
}

function EditorSkeleton() {
  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-4">
          <Skeleton className="h-9 w-9 rounded-md" />
          <Skeleton className="h-6 w-32" />
        </div>
        <Skeleton className="h-9 w-20" />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 flex-1">
        <div className="lg:col-span-2 space-y-4">
          <div className="rounded-xl border bg-card p-6 flex-1">
            <Skeleton className="h-6 w-40 mb-4" />
            <Skeleton className="h-32 w-full" />
          </div>
          <div className="rounded-xl border bg-card p-6">
            <Skeleton className="h-5 w-20 mb-4" />
            <Skeleton className="h-24 w-full" />
          </div>
        </div>
        <div className="rounded-xl border bg-card p-6 h-fit">
          <Skeleton className="h-5 w-16 mb-6" />
          <div className="space-y-4">
            <Skeleton className="h-20 w-full" />
            <Skeleton className="h-40 w-full" />
          </div>
        </div>
      </div>
    </div>
  );
}

function CommandsListSkeleton() {
  return (
    <div className="grid gap-6 lg:grid-cols-[320px,1fr]">
      <div className="rounded-xl border bg-card">
        <div className="p-6 space-y-4">
          <div className="flex items-center justify-between">
            <Skeleton className="h-6 w-24" />
            <Skeleton className="h-9 w-20" />
          </div>
          <Skeleton className="h-9 w-full" />
        </div>
        <div className="p-6 pt-0 space-y-2">
          <Skeleton className="h-16 w-full" />
          <Skeleton className="h-16 w-full" />
          <Skeleton className="h-16 w-full" />
        </div>
      </div>
      <div className="rounded-xl border bg-card p-6 space-y-4">
        <Skeleton className="h-6 w-32" />
        <Skeleton className="h-4 w-64" />
        <div className="space-y-4 pt-4">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-32 w-full" />
          <Skeleton className="h-12 w-full" />
        </div>
      </div>
    </div>
  );
}

function SkillsListSkeleton() {
  return (
    <div className="grid gap-6 lg:grid-cols-[320px,1fr]">
      <div className="rounded-xl border bg-card">
        <div className="p-6 space-y-4">
          <div className="flex items-center justify-between">
            <Skeleton className="h-6 w-16" />
            <Skeleton className="h-9 w-20" />
          </div>
          <Skeleton className="h-4 w-48" />
        </div>
        <div className="p-6 pt-0 space-y-2">
          <Skeleton className="h-16 w-full" />
          <Skeleton className="h-16 w-full" />
          <Skeleton className="h-16 w-full" />
        </div>
      </div>
      <div className="rounded-xl border bg-card p-6 space-y-4">
        <Skeleton className="h-6 w-32" />
        <Skeleton className="h-4 w-64" />
        <div className="space-y-4 pt-4">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-48 w-full" />
          <Skeleton className="h-12 w-full" />
        </div>
      </div>
    </div>
  );
}

export {
  Skeleton,
  CardSkeleton,
  RuleCardSkeleton,
  DashboardSkeleton,
  RulesListSkeleton,
  EditorSkeleton,
  CommandsListSkeleton,
  SkillsListSkeleton,
};
