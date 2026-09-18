import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SelectionIcon` instead.
 *
 * @example
 * ```svelte
 * <Selection color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Selection: Component<IconComponentProps, {}, "">;
type Selection = ReturnType<typeof Selection>;
export default Selection;
