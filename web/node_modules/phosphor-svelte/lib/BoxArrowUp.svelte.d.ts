import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BoxArrowUpIcon` instead.
 *
 * @example
 * ```svelte
 * <BoxArrowUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BoxArrowUp: Component<IconComponentProps, {}, "">;
type BoxArrowUp = ReturnType<typeof BoxArrowUp>;
export default BoxArrowUp;
