import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SplitVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <SplitVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SplitVertical: Component<IconComponentProps, {}, "">;
type SplitVertical = ReturnType<typeof SplitVertical>;
export default SplitVertical;
