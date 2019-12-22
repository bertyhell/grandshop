import React, { FunctionComponent, useState } from 'react';

import { applyModifications, saveImage, selectOriginalImage } from '../../helpers/interop-electron';

import './ControlsSidebar.scss';
import Button from '../../components/Button/Button';
import FormField from '../../components/FormField/FormField';
import Input from '../../components/Input/Input';
import ImageModification from '../../components/ImageModification/ImageModification';
import ColorInput from '../../components/ColorInput/ColorInput';


interface ControlsSidebarProps {
}

const ControlsSidebar: FunctionComponent<ControlsSidebarProps> = () => {
  const [thickness, setThickness] = useState<string>('100');
  const [color, setColor] = useState<string>('#225eff');

  const applyBorder = () => {
    try {
      applyModifications([
        {
          name: 'addBorder',
          arguments: [parseInt(thickness, 10), color],
        }
      ]);
    } catch (err) {
      // show toast
    }
  };

  return (
      <div className="m-controls-sidebar">
        <Button onClick={() => {
          selectOriginalImage({
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
        <div className="m-modification-controls">
        <ImageModification label="Rand 1">
          <FormField label="Dikte" labelFor="thickness">
            <Input id="thickness" type="text" value={thickness} onChange={(value: string) => {
              setThickness(value);
            }}/>
          </FormField>
          <FormField label="Kleur" labelFor="color">
            <ColorInput value={color} onChange={(value: string) => {
              setColor(value);
            }}/>
          </FormField>
          <Button onClick={() => applyBorder()} label="Rand toepassen"/>
        </ImageModification>
        </div>
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
            title: 'Kies de locatie en naam om je afbeelding op te slaan',
          });
        }} label="Afbeelding opslaan als"/>
      </div>
  );
};

export default ControlsSidebar;
