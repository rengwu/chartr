import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `VanIcon` instead.
 *
 * @example
 * ```svelte
 * <Van color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Van: Component<IconComponentProps, {}, "">;
type Van = ReturnType<typeof Van>;
export default Van;
