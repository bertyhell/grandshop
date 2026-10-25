import { ChangeEvent, FunctionComponent, KeyboardEvent } from 'react';

import clsx from 'clsx';

import './Input.scss';

type InputType =
	| 'checkbox'
	| 'date'
	| 'datetime-local'
	| 'email'
	| 'file'
	| 'hidden'
	| 'image'
	| 'month'
	| 'number'
	| 'password'
	| 'radio'
	| 'range'
	| 'reset'
	| 'search'
	| 'submit'
	| 'tel'
	| 'text'
	| 'time'
	| 'url'
	| 'week';

export interface InputProps {
	className?: string;
	id?: string;
	disabled?: boolean;
	placeholder?: string;
	value?: string;
	type?: InputType;
	ariaLabel?: string;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	onChange?: (value: any) => void;
	onBlur?: (event: ChangeEvent<HTMLInputElement>) => void;
	onKeyUp?: (event: KeyboardEvent<HTMLInputElement>) => void;
}

const Input: FunctionComponent<InputProps> = ({
	className,
	id,
	disabled = false,
	placeholder,
	value = '',
	type = 'text',
	ariaLabel = '',
	onChange = () => {},
	onBlur = () => {},
	onKeyUp = () => {},
}) => {
	function onValueChange(event: ChangeEvent<HTMLInputElement>) {
		onChange(event.target.value);
	}

	return (
		<input
			className={clsx('o-input', className)}
			type={type}
			id={id}
			value={value}
			disabled={disabled}
			placeholder={placeholder}
			{...(ariaLabel ? { 'aria-label': ariaLabel } : {})}
			onChange={onValueChange}
			onBlur={onBlur}
			onKeyUp={onKeyUp}
		/>
	);
};

export default Input;
