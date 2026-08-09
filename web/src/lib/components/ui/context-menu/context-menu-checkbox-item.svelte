<script lang="ts">
	import { ContextMenu as ContextMenuPrimitive } from "bits-ui";
	import { Check as CheckIcon } from "phosphor-svelte";
	import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";
	import type { Snippet } from "svelte";

	let {
		ref = $bindable(null),
		checked = $bindable(false),
		indeterminate = $bindable(false),
		class: className,
		inset,
		children: childrenProp,
		...restProps
	}: WithoutChildrenOrChild<ContextMenuPrimitive.CheckboxItemProps> & {
		inset?: boolean;
		children?: Snippet;
	} = $props();
</script>

<ContextMenuPrimitive.CheckboxItem
	bind:ref
	bind:checked
	bind:indeterminate
	data-slot="context-menu-checkbox-item"
	data-inset={inset}
	class={cn(
		"min-h-7 gap-2 rounded-md py-1.5 pr-8 pl-2 text-xs focus:bg-accent focus:text-accent-foreground focus:**:text-accent-foreground data-inset:pl-7.5 [&_svg:not([class*='size-'])]:size-3.5 relative flex cursor-default items-center outline-hidden select-none data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
		className
	)}
	{...restProps}
>
	{#snippet children({ checked })}
		<span class="pointer-events-none absolute right-2 flex items-center justify-center pointer-events-none">
			{#if checked}
				<CheckIcon  />
			{/if}
		</span>
		{@render childrenProp?.()}
	{/snippet}
</ContextMenuPrimitive.CheckboxItem>
