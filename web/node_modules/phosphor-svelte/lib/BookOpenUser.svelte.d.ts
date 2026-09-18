import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BookOpenUserIcon` instead.
 *
 * @example
 * ```svelte
 * <BookOpenUser color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BookOpenUser: Component<IconComponentProps, {}, "">;
type BookOpenUser = ReturnType<typeof BookOpenUser>;
export default BookOpenUser;
