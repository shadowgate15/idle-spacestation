<script lang="ts">
  import { Button } from '$lib/components/ui/button';
</script>

<section data-testid="home-hero" class="mb-8">
  <h2 class="mb-2 text-2xl font-bold tracking-tight text-foreground">Station Command</h2>
  <p class="mb-4 text-muted-foreground">
    Balance oxygen, power, and crew confidence while the station grows one measured upgrade at a
    time.
  </p>
  <Button
    data-testid="primary-action"
    class="bg-primary text-primary-foreground hover:bg-primary/80"
  >
    Review Station Status
  </Button>
</section>

<section
  data-testid="resource-strip"
  class="mb-8 flex gap-6 rounded-lg border border-border bg-card p-4"
>
  <dl class="flex gap-6">
    <div class="flex flex-col">
      <dt class="text-xs tracking-wide text-muted-foreground uppercase">Power Reserve</dt>
      <dd class="text-lg font-bold text-foreground">82%</dd>
    </div>
    <div class="flex flex-col">
      <dt class="text-xs tracking-wide text-muted-foreground uppercase">Oxygen Uptime</dt>
      <dd class="text-lg font-bold text-foreground">99.2%</dd>
    </div>
    <div class="flex flex-col">
      <dt class="text-xs tracking-wide text-muted-foreground uppercase">Crew Morale</dt>
      <dd class="text-lg font-bold text-foreground">Stable</dd>
    </div>
  </dl>
</section>

<div class="grid gap-6 lg:grid-cols-2">
  <div class="flex flex-col gap-6">
    <section
      id="overview-panel"
      data-testid="overview-panel"
      class="rounded-lg border border-border bg-card p-4"
    >
      <h2 class="mb-3 text-base font-semibold text-foreground">Operational Snapshot</h2>
      <ul class="space-y-2 text-sm text-muted-foreground">
        <li>Solar lattice holding steady across the sunline.</li>
        <li>Hydroponics cycle recovered after the last coolant swap.</li>
        <li>Freighter berth clear for the next supply approach.</li>
      </ul>
    </section>

    <section
      id="alerts-panel"
      data-testid="alerts-panel"
      class="rounded-lg border border-border bg-card p-4"
    >
      <h2 class="mb-3 text-base font-semibold text-foreground">Command Alerts</h2>
      <ul class="space-y-2 text-sm text-muted-foreground">
        <li>Cooling loop efficiency dipped 3% overnight.</li>
        <li>Cargo drone maintenance window opens in 02:15.</li>
      </ul>
    </section>
  </div>

  <section
    id="systems-panel"
    data-testid="systems-panel"
    class="rounded-lg border border-border bg-card p-4"
  >
    <h2 class="mb-3 text-base font-semibold text-foreground">Priority Systems</h2>
    <div class="space-y-4">
      <article class="flex flex-col gap-1">
        <h3 class="text-sm font-medium text-foreground">Reactor</h3>
        <p class="text-sm text-muted-foreground">
          Output locked at 4.8 GW with no variance outside tolerance.
        </p>
      </article>
      <article class="flex flex-col gap-1">
        <h3 class="text-sm font-medium text-foreground">Life Support</h3>
        <p class="text-sm text-muted-foreground">
          Scrubbers cycling cleanly across all habitation rings.
        </p>
      </article>
      <article class="flex flex-col gap-1">
        <h3 class="text-sm font-medium text-foreground">Docking Grid</h3>
        <p class="text-sm text-muted-foreground">
          Approach beacons aligned and berth traffic remains orderly.
        </p>
      </article>
    </div>
  </section>
</div>
