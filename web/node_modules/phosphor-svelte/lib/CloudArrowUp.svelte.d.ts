import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CloudArrowUpIcon` instead.
 *
 * @example
 * ```svelte
 * <CloudArrowUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CloudArrowUp: Component<IconComponentProps, {}, "">;
type CloudArrowUp = ReturnType<typeof CloudArrowUp>;
export default CloudArrowUp;
