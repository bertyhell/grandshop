import classnames from 'classnames';
import React, { FunctionComponent } from 'react';

import './Accordion.scss';
import Button from '../Button/Button';

export interface AccordionProps {
  className?: string;
  isOpen?: boolean;
  onToggle?: () => void;
  onDelete?: () => void;
  title?: string;
}

export const Accordion: FunctionComponent<AccordionProps> = ({
                                                               children,
                                                               className,
                                                               isOpen = true,
                                                               onToggle,
                                                               onDelete,
                                                               title
                                                             }) => {
  return (
      <div className={classnames(className, 'c-accordion', { 'c-accordion-closed': !isOpen })}>
        <div
            className={classnames('c-accordion-header', { 'u-clickable': onToggle })}
            onClick={onToggle}
        >
          <div className="c-accordion-header-title">{title}</div>
          <div className="c-button-wrapper">
            <Button label="Verwijder" onClick={onDelete} block={false} type="secondary"/>
          </div>
        </div>
        <div className="c-accordion-body">
          <div className="c-accordion-item">{children}</div>
        </div>
      </div>
  );
};
