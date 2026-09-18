import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `KanbanIcon` instead.
 *
 * @example
 * ```svelte
 * <Kanban color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Kanban: Component<IconComponentProps, {}, "">;
type Kanban = ReturnType<typeof Kanban>;
export default Kanban;
