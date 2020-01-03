import React, { FunctionComponent, useEffect } from 'react';
import classnames from 'classnames';

import './Sepia.scss';
import ImageModification from '../../../components/ImageModification/ImageModification';
import { ImageModificationInfo } from '../../types';

interface SepiaProps {
  className?: string;
  title: string;
  onApply: (modification: ImageModificationInfo) => void,
  isOpen: boolean,
  onToggle: () => void,
  onDelete: () => void,
}

const Sepia: FunctionComponent<SepiaProps> = ({
                                                className,
                                                title,
                                                onApply,
                                                isOpen,
                                                onToggle,
                                                onDelete
                                              }) => {
  useEffect(() => {
    onApply({
          name: 'sepia',
          arguments: []
        }
    );
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  return (
      <ImageModification
          label={title}
          className={classnames('c-sepia', className)}
          isOpen={isOpen}
          onToggle={onToggle}
          onDelete={onDelete}
      />
  );
};

export default Sepia;
