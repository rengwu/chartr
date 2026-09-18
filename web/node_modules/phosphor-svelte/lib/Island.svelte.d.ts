import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `IslandIcon` instead.
 *
 * @example
 * ```svelte
 * <Island color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Island: Component<IconComponentProps, {}, "">;
type Island = ReturnType<typeof Island>;
export default Island;
