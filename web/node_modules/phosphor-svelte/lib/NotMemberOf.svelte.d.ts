import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NotMemberOfIcon` instead.
 *
 * @example
 * ```svelte
 * <NotMemberOf color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NotMemberOf: Component<IconComponentProps, {}, "">;
type NotMemberOf = ReturnType<typeof NotMemberOf>;
export default NotMemberOf;
