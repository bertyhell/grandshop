import React, { FunctionComponent, useEffect, useReducer, useState } from 'react';
import getUuid from 'uuid/v1';
import compact from 'lodash-es/compact';
import { connect } from 'react-redux';
import { useDebounce } from 'use-debounce';

import { applyModifications, saveImage, openImage } from '../../helpers/interop-electron';
import Button from '../../components/Button/Button';
import { AppState } from '../../store/types';
import { selectModifiedImage, selectOriginalImage } from '../../store/selectors';
import { Select, SelectOption } from '../../components/Select/Select';
import AddBorder from '../../components/ImageModifications/AddBorder/AddBorder';
import { format } from '../../helpers/format';

import './ControlsSidebar.scss';
import get from 'lodash-es/get';
import usePrevious from 'use-previous';
import Sepia from '../../components/ImageModifications/Sepia/Sepia';
import Brightness from '../../components/ImageModifications/Brightness/Brightness';
import { ImageModificationInfo } from '../../shared/types';

interface ControlsSidebarProps {
  originalImageUrl: string | null;
  modifiedImageUrl: string | null;
}

interface ActiveModifier {
  name: string;
  title: string;
  info: ImageModificationInfo | null;
  isOpen: boolean;
  key: string;
}

const ControlsSidebar: FunctionComponent<ControlsSidebarProps> = ({
                                                                    originalImageUrl,
                                                                    modifiedImageUrl
                                                                  }) => {
  const [selectedOption, setSelectedOption] = useState<string>('');

  const IMAGE_MODIFIER_OPTIONS: SelectOption[] = [
    {
      label: "Kies een bewerking",
      value: "",
      disabled: true
    },
    {
      label: "Voeg een gekleurde rand toe",
      value: "addBorder"
    },
    {
      label: "Voeg een sepia effect toe",
      value: "sepia"
    },
    {
      label: "Pas de helderheid aan",
      value: "brightness"
    }
  ];

  const IMAGE_MODIFIER_TITLES: { [modifierName: string]: string } = {
    addBorder: "Rand {0}",
    sepia: "Sepia {0}",
    brightness: "Helderheid {0}",
  };

  type ActiveModifierAction =
      | { type: 'ADD', name: string }
      | { type: 'UPDATE', info: ImageModificationInfo, index: number }
      | { type: 'TOGGLE', index: number }
      | { type: 'DELETE', index: number }

  const getModifierTitle = (activeModifiers: ActiveModifier[], modifierName: string): string => {
    return format(IMAGE_MODIFIER_TITLES[modifierName], activeModifiers.filter(mod => mod.name === modifierName).length + 1);
  };

  function activeModifiersReducer(activeModifiers: ActiveModifier[], action: ActiveModifierAction): ActiveModifier[] {
    let modifiers: ActiveModifier[] | undefined;
    let key = getUuid();
    switch (action.type) {
      case 'ADD':
        const title: string = getModifierTitle(activeModifiers, action.name);
        const newModifier = {
          title,
          info: null,
          name: action.name,
          isOpen: true,
          key
        };
        return [
          ...activeModifiers.map(mod => {
            mod.isOpen = false; // Close all previous modifiers when adding a new one
            return mod;
          }),
          newModifier
        ];

      case 'UPDATE':
        modifiers = [...activeModifiers];
        modifiers[action.index] = {
          ...modifiers[action.index],
          info: action.info
        };
        return modifiers;

      case 'TOGGLE':
        console.log('toggle before ' + get(activeModifiers, `[${action.index}].title`, null) + ' ' + get(activeModifiers, `[${action.index}].isOpen`, null));
        const originalIsOpen: boolean = get(activeModifiers, `[${action.index}].isOpen`, false);
        modifiers = [
          ...activeModifiers.map(mod => {
            mod.isOpen = false; // Close all previous modifiers when adding a new one
            return mod;
          })
        ];
        modifiers[action.index] = {
          ...modifiers[action.index],
          isOpen: !originalIsOpen
        };
        console.log('toggle after ' + modifiers[action.index].title + ' ' + modifiers[action.index].isOpen);
        return modifiers;

      case 'DELETE':
        modifiers = [
          ...activeModifiers
        ];
        modifiers.splice(action.index, 1);
        return modifiers;
    }
  }

  const [activeModifiers, modifyActiveModifiers] = useReducer<React.Reducer<ActiveModifier[], ActiveModifierAction>>(activeModifiersReducer, []);
  const [debouncedActiveModifiers] = useDebounce(activeModifiers, 500);
  const previousActiveModifiers = usePrevious<ActiveModifier[] | null>(activeModifiers);

  useEffect(() => {
    const modifications: ImageModificationInfo[] = compact(debouncedActiveModifiers.map((m: ActiveModifier) => m.info));

    // check previous state to see if we need to run the applyModifications function because all modifiers were removed
    if (modifications && (modifications.length || (previousActiveModifiers && previousActiveModifiers.length !== modifications.length))) {
      applyModifications(modifications); // TODO add spinner while modifications are in progress
    }
  }, [debouncedActiveModifiers]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleApplyModifier = (modifierInfo: ImageModificationInfo, index: number) => {
    modifyActiveModifiers({ type: 'UPDATE', info: modifierInfo, index });
  };

  const handleToggleModifier = (index: number) => {
    modifyActiveModifiers({ type: 'TOGGLE', index });
  };

  const handleDeleteModifier = (index: number) => {
    modifyActiveModifiers({ type: 'DELETE', index });
  };

  const handleModifierSelectionChanged = (name: string) => {
    modifyActiveModifiers({ type: 'ADD', name });
    setSelectedOption('');
  };

  return (
      <div className="m-controls-sidebar">
        <div className="m-button-wrapper">
          <Button onClick={() => {
            openImage({
              buttonLabel: "Openen",
              // defaultPath // TODO default to the users images folder
              filters: [
                {
                  name: 'Alle bestanden',
                  extensions: ['*']
                },
                {
                  name: 'Afbeeldingen',
                  extensions: ['jpg', 'png', 'gif', 'tif', 'bmp', 'jpeg', 'webp']
                }
              ],
              title: 'Selecteer de afbeelding die je wil bewerken',
              properties: [
                // 'multiSelections', // TODO add multi image support
                'openFile'
              ]
            });
          }} label="Afbeelding openen"/>
        </div>
        {originalImageUrl &&
        <>
          <div className="m-button-wrapper">
            <Select
              options={IMAGE_MODIFIER_OPTIONS}
              onChange={handleModifierSelectionChanged}
              value={selectedOption}
            />
          </div>
          <div className="m-modification-controls">
            {activeModifiers.map((m: ActiveModifier, index: number) => {
              switch (m.name) {
                case 'addBorder':
                  return <AddBorder
                      title={m.title}
                      onApply={(info) => handleApplyModifier(info, index)}
                      isOpen={m.isOpen}
                      onToggle={() => handleToggleModifier(index)}
                      onDelete={() => handleDeleteModifier(index)}
                      key={m.key}/>;
                case 'sepia':
                  return <Sepia
                      title={m.title}
                      onApply={(info) => handleApplyModifier(info, index)}
                      isOpen={m.isOpen}
                      onToggle={() => handleToggleModifier(index)}
                      onDelete={() => handleDeleteModifier(index)}
                      key={m.key}/>;
                case 'brightness':
                  return <Brightness
                      title={m.title}
                      onApply={(info) => handleApplyModifier(info, index)}
                      isOpen={m.isOpen}
                      onToggle={() => handleToggleModifier(index)}
                      onDelete={() => handleDeleteModifier(index)}
                      key={m.key}/>;
              }
              return null;
            })}
          </div>
        </>
        }
        {modifiedImageUrl &&
        <div className="m-button-wrapper">
          <Button className="c-save-button" onClick={() => {
            saveImage({
              buttonLabel: "Opslaan",
              // defaultPath // TODO default to the users images folder
              filters: [
                {
                  name: 'Alle bestanden',
                  extensions: ['*']
                },
                {
                  name: 'Afbeeldingen',
                  extensions: ['jpg', 'png', 'gif', 'tif', 'bmp', 'jpeg', 'webp']
                }
              ],
              title: 'Kies de locatie en naam om je afbeelding op te slaan'
            });
          }} label="Afbeelding opslaan als"/>
        </div>
        }
      </div>
  );
};

const mapStateToProps = (state: { app: AppState }) => ({
  originalImageUrl: selectOriginalImage(state),
  modifiedImageUrl: selectModifiedImage(state)
});

export default connect(mapStateToProps)(ControlsSidebar);
